use crate::header::{parse_header, Header, HEADER_SIZE};
use js_sys::{Array, Uint8Array};
use quircs::Quirc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, BlobPropertyBag, CanvasRenderingContext2d, HtmlAnchorElement, HtmlCanvasElement,
    HtmlVideoElement, MediaStream, Url,
};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::utils::window;

type FnCB = Box<dyn FnMut(JsValue)>;

pub struct RecvPage {
    link: ComponentLink<RecvPage>,
    start: bool,
    timer_id: i32,
    canvas_element: NodeRef,
    video_element: NodeRef,
    recv_ready: bool,
    received_bytes: usize,
    received: Vec<Uint8Array>,
}

pub enum Msg {
    Start,
    InitVideo(MediaStream),
    VideoStart,
    Enqueue,
    Recognized(Header, Uint8Array),
}

impl RecvPage {
    fn get_display_media_callback(link: ComponentLink<RecvPage>, s: MediaStream) {
        link.send_message(Msg::InitVideo(s.clone()));
    }

    fn process(
        link: ComponentLink<RecvPage>,
        video: HtmlVideoElement,
        context: CanvasRenderingContext2d,
        w: f64,
        h: f64,
    ) {
        context
            .draw_image_with_html_video_element(&video, 0.0, 0.0)
            .unwrap();
        let img = context.get_image_data(0.0, 0.0, w, h).unwrap();
        let d = img.data().0;
        let hi = h as i32;
        let wi = w as i32;
        let mut gs: Vec<u8> = Vec::with_capacity((hi as usize) * (wi as usize));
        gs.resize(gs.capacity(), 0);
        for y in 0..hi {
            for x in 0..wi {
                let j = (y * wi + x) as usize;
                let i = j * 4;
                let (r, g, b) = (d[i], d[i + 1], d[i + 2]);
                gs[j] = ((0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32).clamp(0.0, 1.0)
                    * 255.0) as u8;
            }
        }
        let mut decoder = Quirc::default();
        let codes = decoder.identify(w as usize, hi as usize, &gs);
        for code in codes {
            let code = code.expect("failed to extract qr code");
            match code.decode() {
                Ok(decoded) => {
                    let d = decoded.payload;
                    let header = parse_header(&d[..]);
                    let buf = Uint8Array::new_with_length(header.size as u32);
                    buf.copy_from(&d[HEADER_SIZE..HEADER_SIZE + header.size as usize]);
                    link.send_message(Msg::Recognized(header, buf));
                }
                Err(e) => {
                    ConsoleService::log(format!("ERROR: {:?}", e).as_ref());
                }
            }
        }
        link.send_message(Msg::Enqueue);
    }

    fn start_download(&mut self) {
        let array = Array::new_with_length(self.received.len() as u32);
        for i in 0..self.received.len() {
            let tmp: &JsValue = self.received[i].as_ref();
            array.set(i as u32, tmp.clone());
        }
        let mut props = BlobPropertyBag::new();
        props.type_("octet/stream");
        let blob = Blob::new_with_blob_sequence_and_options(array.as_ref(), &props).unwrap();
        let url = Url::create_object_url_with_blob(blob.as_ref()).unwrap();

        let doc = window().document().unwrap();
        let a = doc
            .create_element("a")
            .unwrap()
            .dyn_into::<HtmlAnchorElement>()
            .unwrap();
        a.set_href(&url);
        a.click();
        Url::revoke_object_url(&url).unwrap();
    }
}

impl Component for RecvPage {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            start: false,
            timer_id: -1,
            canvas_element: NodeRef::default(),
            video_element: NodeRef::default(),
            recv_ready: false,
            received_bytes: 0,
            received: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start => {
                self.start = true;
                self.recv_ready = false;
                self.received_bytes = 0;
                self.received.clear();
                let link = self.link.clone();
                let cb = Closure::wrap(Box::new(move |v: JsValue| {
                    Self::get_display_media_callback(
                        link.clone(),
                        v.dyn_into::<MediaStream>().unwrap(),
                    );
                }) as FnCB);
                let _ = window()
                    .navigator()
                    .media_devices()
                    .unwrap()
                    .get_display_media()
                    .unwrap()
                    .then(&cb);
                cb.forget();
            }
            Msg::InitVideo(s) => {
                let video = self.video_element.cast::<HtmlVideoElement>().unwrap();
                video.set_src_object(Some(&s));
            }
            Msg::VideoStart | Msg::Enqueue => {
                let link = self.link.clone();
                let video = self.video_element.cast::<HtmlVideoElement>().unwrap();
                let canvas = self.canvas_element.cast::<HtmlCanvasElement>().unwrap();
                let (vw, vh) = (video.video_width() as f64, video.video_height() as f64);
                match msg {
                    Msg::VideoStart => {
                        canvas.set_height(vh as u32);
                        canvas.set_width(vw as u32);
                    }
                    _ => {
                        if self.timer_id < 0 {
                            return false;
                        }
                    }
                }
                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap();
                let cb = Closure::wrap(Box::new(move || {
                    RecvPage::process(link.clone(), video.clone(), context.clone(), vw, vh);
                }) as Box<dyn Fn()>);
                self.timer_id = window()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        1,
                    )
                    .unwrap();
                cb.forget();
            }
            Msg::Recognized(header, buf) => {
                if (header.seq == 0 && buf.length() == 0) || self.timer_id < 0 {
                    if !self.recv_ready {
                        self.recv_ready = true;
                        return true;
                    }
                    return false;
                }
                if self.received.len() < header.seq as usize {
                    window().clear_timeout_with_handle(self.timer_id);
                    self.timer_id = -1;
                    let _ = window().alert_with_message("受信に失敗しました: シーケンス番号ずれ");
                    window().location().reload().unwrap();
                    return false;
                }
                if self.received.len() == header.seq as usize {
                    self.received_bytes += buf.byte_length() as usize;
                    self.received.push(buf);
                    return true;
                }
                if buf.length() == 0 {
                    // EOF
                    window().clear_timeout_with_handle(self.timer_id);
                    self.timer_id = -1;
                    self.start = false;
                    self.start_download();
                    return true;
                }
                return false;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn destroy(&mut self) {
        if self.timer_id > 0 {
            window().clear_timeout_with_handle(self.timer_id);
            self.timer_id = -1;
        }
    }

    fn view(&self) -> Html {
        let onclick = self.link.callback(|_| Msg::Start);
        let onplay = self.link.callback(|_| Msg::VideoStart);
        let msg = if !self.recv_ready {
            "スクリーンキャプチャ先のQRコードを認識できません。".to_string()
        } else if self.received_bytes == 0 {
            "送信側のデータ送出を待機中...".to_string()
        } else {
            format!(
                "{}バイト受信済み. 送受信QRコード数:{}",
                self.received_bytes,
                self.received.len()
            )
        };
        html! {
            <div class="recv-page">
                <div>
                    <button onclick={onclick} disabled={self.start}>{ "受信開始" }</button>
                </div>
                <div>{ if self.start { &msg } else { "" } }</div>
                <video ref=self.video_element.clone() muted={true} autoplay="true" onplay={onplay} style="display: none" />
                <canvas ref=self.canvas_element.clone() style="display: none" />
            </div>
        }
    }
}
