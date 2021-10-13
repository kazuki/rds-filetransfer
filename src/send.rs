use qrcode::{QrCode, Color, EcLevel};
use yew::prelude::*;
use yew::utils::window;
use web_sys::{HtmlElement, HtmlCanvasElement, HtmlInputElement, File, FileReader, Blob};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::header::{Header, HEADER_SIZE, build_header};

const DEFAULT_BLOCK_SIZE: u16 = 2951; //2953;
const DEFAULT_INTERVAL: u16 = 1000;

pub struct SendPage {
    link: ComponentLink<SendPage>,
    block_size: u16,
    send_interval: u16,
    pixel_size: u8,
    data: Vec<u8>,
    canvas: NodeRef,
    file: Option<File>,
    file_inflight: Option<Blob>,
    timeout_id: i32,
    read_offset: u64,
}

#[derive(Debug)]
pub enum Msg {
    Start(File),
    LoadFile(FileReader, ArrayBuffer),
    UpdateBlockSize(u16),
    UpdateInterval(u16),
}

impl SendPage {
    fn render_qrcode(&self) -> Result<(), ()> {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().ok_or(())?;
        let context = canvas.get_context("2d").map_err(|_| ())?.ok_or(())?.dyn_into::<web_sys::CanvasRenderingContext2d>().map_err(|_| ())?;
        let code = QrCode::with_error_correction_level(&self.data, EcLevel::L).map_err(|_| ())?;
        let colors = code.to_colors();
        let size = (colors.len() as f64).sqrt() as u32;
        let canvas_size = size * self.pixel_size as u32;
        let rect_size = self.pixel_size as f64;
        if canvas.height() != canvas_size {
            canvas.set_height(canvas_size);
        }
        if canvas.width() != canvas_size {
            canvas.set_width(canvas_size);
        }

        context.clear_rect(0.0, 0.0, canvas_size as f64, canvas_size as f64);
        for y in 0..size {
            for x in 0..size {
                if colors[(y * size + x) as usize] == Color::Light {
                    continue;
                }
                context.fill_rect(x as f64 * rect_size, y as f64 * rect_size, rect_size, rect_size);
            }
        }

        Ok(())
    }

    fn start(&mut self, f: File) {
        self.file = Some(f);
        self.read_offset = 0;
        let reader = FileReader::new().unwrap();
        let reader2 = reader.clone();
        let cur_offset = self.read_offset as f64;
        let read_size = (self.block_size - HEADER_SIZE as u16) as f64;
        let read_size2 = read_size;
        let link2 = self.link.clone();
        let cb = Closure::wrap(Box::new(move || {
            if reader2.result().is_err() {
                return;
            }
            if let Ok(buf) = reader2.result().unwrap().dyn_into::<ArrayBuffer>() {
                if buf.byte_length() == 0 {
                    return;
                }
                link2.send_message(Msg::LoadFile(reader2.clone(), buf));
            }
        }) as Box<dyn Fn()>);
        reader.set_onload(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        self.file_inflight = Some(self.file.as_ref().unwrap().slice_with_f64_and_f64(
            cur_offset, cur_offset + read_size2).unwrap());
        reader.read_as_array_buffer(self.file_inflight.as_ref().unwrap()).unwrap();
    }
}

impl Component for SendPage {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut data = Vec::new();
        data.resize(DEFAULT_BLOCK_SIZE as usize, 0);
        Self {
            link,
            block_size: DEFAULT_BLOCK_SIZE,
            send_interval: DEFAULT_INTERVAL,
            pixel_size: 4,
            data,
            canvas: NodeRef::default(),
            file: None,
            file_inflight: None,
            timeout_id: -1,
            read_offset: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start(f) => {
                self.start(f);
            },
            Msg::LoadFile(reader, buf) => {
                if buf.byte_length() == 0 {
                    self.file_inflight = None;
                    return true;
                }
                let seq = (self.read_offset / (self.block_size - HEADER_SIZE as u16) as u64) as u32;
                self.read_offset += buf.byte_length() as u64;
                self.file_inflight = Some(self.file.as_ref().unwrap().slice_with_f64_and_f64(
                    self.read_offset as f64,
                    (self.read_offset + (self.block_size - HEADER_SIZE as u16) as u64) as f64).unwrap());

                Uint8Array::new(&buf).copy_to(&mut self.data[HEADER_SIZE..]);
                build_header(Header { seq, size: buf.byte_length() as u16 }, &mut self.data[..]);
                self.render_qrcode().unwrap();

                let reader2 = reader.clone();
                let file_inflight2 = self.file_inflight.clone();
                let cb = Closure::wrap(Box::new(move || {
                    reader2.read_as_array_buffer(file_inflight2.as_ref().unwrap()).unwrap();
                }) as Box<dyn Fn()>);
                window().set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    self.send_interval as i32
                ).unwrap();
                cb.forget();
            },
            Msg::UpdateBlockSize(v) => {
                self.block_size = v;
                self.data.resize(v as usize, 0);
                self.render_qrcode().unwrap();
            },
            Msg::UpdateInterval(v) => self.send_interval = v,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.render_qrcode().unwrap();
        }
    }

    fn destroy(&mut self) {
        if self.timeout_id > 0 {
            window().clear_timeout_with_handle(self.timeout_id);
            self.timeout_id = -1;
        }
    }

    fn view(&self) -> Html {
        let (block_size, send_interval) = (self.block_size, self.send_interval);
        let oninput = self.link.batch_callback(move |e: InputData| {
            let v = e.value.parse::<u16>();
            if let Some(tgt) = e.event.target() {
                if let Some(element) = tgt.dyn_ref::<HtmlElement>() {
                    if element.id() == "block-size" {
                        return Some(Msg::UpdateBlockSize(v.unwrap_or(block_size)));
                    } else {
                        return Some(Msg::UpdateInterval(v.unwrap_or(send_interval)));
                    }
                }
            }
            None
        });
        let onstart = self.link.batch_callback(|e: InputData| {
            if let Some(tgt) = e.event.target() {
                if let Some(element) = tgt.dyn_ref::<HtmlInputElement>() {
                    if let Some(files) = element.files() {
                        if let Some(f) = files.item(0) {
                            return Some(Msg::Start(f));
                        }
                    }
                }
            }
            None
        });
        let in_progress = self.file.is_some();

        html! {
            <div class="send-page">
                <div class="header">
                    <div class="form-block" style="display: none">
                        <label for="block-size">{ "一度に送信するデータ量[B]:"}</label>
                        <input type="number" id="block-size" value={self.block_size.to_string()} size="3" oninput={&oninput} max="2953" />
                    </div>
                    <div class="form-block">
                        <label for="interval">{ "送信間隔[ms]:"}</label>
                        <input type="number" id="interval" value={self.send_interval.to_string()} size="3" oninput={&oninput} disabled={in_progress} />
                    </div>
                    <input type="file" id="input-file" oninput={onstart} disabled={in_progress} />
                    <label for="input-file" class="send-file-label" disabled={in_progress}>{ "ファイルを選んで送信を開始する" }</label>
                </div>
                <canvas id="qrcode" ref=self.canvas.clone() />
            </div>
        }
    }
}
