use js_sys::{ArrayBuffer, Uint8Array};
use qrcode::{Color, EcLevel, QrCode, Version};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Blob, File, FileReader, HtmlCanvasElement, HtmlElement, HtmlInputElement, InputEvent,
};
use yew::prelude::*;
use yew::utils::window;

use crate::header::{build_header, Header, HEADER_SIZE};

const DEFAULT_VERSION: Version = Version::Normal(40);
const DEFAULT_INTERVAL: u16 = 500;
const DEFAULT_EC_LEVEL: EcLevel = EcLevel::L;
const DEFAULT_PIXEL_SIZE: u8 = 5;
const EC_LEVEL_TABLE: [&str; 4] = ["L", "M", "Q", "H"];

pub struct SendPage {
    scale: f64,
    link: ComponentLink<SendPage>,
    version: Version,
    ec_level: EcLevel,
    block_size: u16,
    send_interval: u16,
    pixel_size: u8,
    data: Vec<u8>,
    canvas: NodeRef,
    file: Option<File>,
    file_inflight: Option<Blob>,
    timeout_id: i32,
    read_offset: u64,
    cache_context_attrs: JsValue,
    cache_black_str: JsValue,
    cache_white_str: JsValue,
}

#[derive(Debug)]
pub enum Msg {
    Start(File),
    LoadFile(FileReader, ArrayBuffer),
    Final,
    UpdateVersion(Version),
    UpdateECLevel(EcLevel),
    UpdateInterval(u16),
    UpdateCellSize(u8),
}

impl SendPage {
    fn render_qrcode(&self) -> Result<(), ()> {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().ok_or(())?;
        let context = canvas
            .get_context_with_context_options("2d", &self.cache_context_attrs)
            .map_err(|_| ())?
            .ok_or(())?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| ())?;
        let code =
            QrCode::with_error_correction_level(&self.data, self.ec_level).map_err(|_| ())?;
        let colors = code.to_colors();
        let size = (colors.len() as f64).sqrt() as u32;
        let canvas_size = size * self.pixel_size as u32;
        let rect_size = self.pixel_size as f64;
        let scaled_canvas_size = (canvas_size as f64 * self.scale) as u32;

        canvas
            .style()
            .set_property("height", format!("{}px", canvas_size).as_ref())
            .unwrap();
        canvas.set_height(scaled_canvas_size);
        canvas
            .style()
            .set_property("width", format!("{}px", canvas_size).as_ref())
            .unwrap();
        canvas.set_width(scaled_canvas_size);
        context.scale(self.scale, self.scale).unwrap();
        context.set_image_smoothing_enabled(false);

        context.set_fill_style(&self.cache_white_str);
        context.fill_rect(0.0, 0.0, canvas_size as f64, canvas_size as f64);
        context.set_fill_style(&self.cache_black_str);

        for y in 0..size {
            for x in 0..size {
                if colors[(y * size + x) as usize] == Color::Light {
                    continue;
                }
                context.fill_rect(
                    x as f64 * rect_size,
                    y as f64 * rect_size,
                    rect_size,
                    rect_size,
                );
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
                    link2.send_message(Msg::Final);
                    return;
                }
                link2.send_message(Msg::LoadFile(reader2.clone(), buf));
            }
        }) as Box<dyn Fn()>);
        reader.set_onload(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        self.file_inflight = Some(
            self.file
                .as_ref()
                .unwrap()
                .slice_with_f64_and_f64(cur_offset, cur_offset + read_size2)
                .unwrap(),
        );
        reader
            .read_as_array_buffer(self.file_inflight.as_ref().unwrap())
            .unwrap();
    }

    fn update_block_size_only(&mut self) {
        let v_idx = match self.version {
            Version::Normal(v) => (v - 1) as usize,
            _ => return,
        };
        self.block_size = BINARY_SIZE_TABLE[v_idx][self.ec_level as usize];
        self.data.resize(self.block_size as usize, 0);
    }

    fn update_block_size(&mut self) {
        self.update_block_size_only();
        self.render_qrcode().unwrap();
    }
}

impl Component for SendPage {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let attrs = Context2DAttributes { alpha: false };
        let context_attrs = JsValue::from_serde(&attrs).unwrap();
        let mut ret = Self {
            scale: window().device_pixel_ratio(),
            link,
            version: DEFAULT_VERSION,
            ec_level: DEFAULT_EC_LEVEL,
            block_size: 0,
            send_interval: DEFAULT_INTERVAL,
            pixel_size: DEFAULT_PIXEL_SIZE,
            data: Vec::new(),
            canvas: NodeRef::default(),
            file: None,
            file_inflight: None,
            timeout_id: -1,
            read_offset: 0,
            cache_context_attrs: context_attrs,
            cache_black_str: JsValue::from_str("black"),
            cache_white_str: JsValue::from_str("white"),
        };
        ret.update_block_size_only();
        ret
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Start(f) => {
                self.start(f);
            }
            Msg::LoadFile(reader, buf) => {
                let read_size = buf.byte_length();
                if read_size == 0 {
                    self.file_inflight = None;
                    return true;
                }
                let seq = (self.read_offset / (self.block_size - HEADER_SIZE as u16) as u64) as u32;
                self.read_offset += read_size as u64;
                self.file_inflight = Some(
                    self.file
                        .as_ref()
                        .unwrap()
                        .slice_with_f64_and_f64(
                            self.read_offset as f64,
                            (self.read_offset + (self.block_size - HEADER_SIZE as u16) as u64)
                                as f64,
                        )
                        .unwrap(),
                );

                Uint8Array::new(&buf)
                    .copy_to(&mut self.data[HEADER_SIZE..HEADER_SIZE + read_size as usize]);
                build_header(
                    Header {
                        seq,
                        size: read_size as u16,
                    },
                    &mut self.data[..],
                );
                self.render_qrcode().unwrap();

                let file_inflight2 = self.file_inflight.clone();
                let cb = Closure::wrap(Box::new(move || {
                    reader
                        .read_as_array_buffer(file_inflight2.as_ref().unwrap())
                        .unwrap();
                }) as Box<dyn Fn()>);
                window()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        self.send_interval as i32,
                    )
                    .unwrap();
                cb.forget();
            }
            Msg::Final => {
                let seq =
                    (self.read_offset / (self.block_size - HEADER_SIZE as u16) as u64) as u32 + 1;
                self.data.clear();
                self.data.resize(self.data.capacity(), 0);
                build_header(Header { seq, size: 0 }, &mut self.data[..]);
                self.render_qrcode().unwrap();
            }
            Msg::UpdateVersion(v) => {
                self.version = v;
                self.update_block_size();
            }
            Msg::UpdateECLevel(v) => {
                self.ec_level = v;
                self.update_block_size();
            }
            Msg::UpdateCellSize(v) => {
                self.pixel_size = v;
                self.render_qrcode().unwrap();
            }
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
        let send_interval = self.send_interval;
        let oninput = self.link.batch_callback(move |e: InputData| {
            let v = e.value.parse::<u16>();
            if let Some(id) = get_event_target_element_id(e.event) {
                if id == "interval" {
                    return Some(Msg::UpdateInterval(v.unwrap_or(send_interval)));
                }
            }
            None
        });
        let onchange = self.link.batch_callback(move |e: ChangeData| {
            if let ChangeData::Select(element) = e {
                let v = element.value().parse::<u16>().unwrap();
                match element.id().as_str() {
                    "version" => {
                        return Some(Msg::UpdateVersion(Version::Normal(v as i16)));
                    }
                    "ec" => {
                        if let Some(l) = to_ec_level(v) {
                            return Some(Msg::UpdateECLevel(l));
                        }
                    }
                    "pixel" => {
                        return Some(Msg::UpdateCellSize(v as u8));
                    }
                    _ => {}
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
        let selected_version = if let Version::Normal(v) = self.version {
            v
        } else {
            0
        };

        html! {
            <div class="send-page">
                <div class="header">
                    <div class="form-block">
                        <label for="version">{ "バージョン:"}</label>
                        <select id="version" disabled={in_progress} onchange={&onchange}>
                        {
                            for (1..=40).map(|version| {
                                let vs = version.to_string();
                                html!{ <option value={ vs.clone() } selected={ selected_version == version }>{ vs }</option> }
                            })
                        }
                        </select>
                    </div>
                    <div class="form-block">
                        <label for="ec">{ "EC:"}</label>
                        <select id="ec" disabled={in_progress} onchange={&onchange}>
                        {
                            for (0..EC_LEVEL_TABLE.len()).map(|ecl| {
                                html!{ <option value={ ecl.to_string() } selected={ self.ec_level as usize == ecl }>{ EC_LEVEL_TABLE[ecl] }</option> }
                            })
                        }
                        </select>
                    </div>
                    <div class="form-block">
                        <label for="pixel">{ "pixels/cell:"}</label>
                        <select id="pixel" disabled={in_progress} onchange={&onchange}>
                        {
                            for (3..=16).map(|s| {
                                html!{ <option value={ s.to_string() } selected={ self.pixel_size as usize == s }>{ s.to_string() }</option> }
                            })
                        }
                        </select>
                    </div>
                    <div class="form-block">
                        <label for="interval">{ "送信間隔[ms]:"}</label>
                        <input type="number" id="interval" value={self.send_interval.to_string()} oninput={&oninput} disabled={in_progress} />
                    </div>
                    <input type="file" id="input-file" oninput={onstart} disabled={in_progress} />
                    <label for="input-file" class="send-file-label" disabled={in_progress}>{ "ファイルを選んで送信を開始する" }</label>
                </div>
                <canvas id="qrcode" ref=self.canvas.clone() />
            </div>
        }
    }
}

fn to_ec_level(v: u16) -> Option<EcLevel> {
    match v {
        0 => Some(EcLevel::L),
        1 => Some(EcLevel::M),
        2 => Some(EcLevel::Q),
        3 => Some(EcLevel::H),
        _ => None,
    }
}

fn get_event_target_element_id(event: InputEvent) -> Option<String> {
    if let Some(tgt) = event.target() {
        if let Some(element) = tgt.dyn_ref::<HtmlElement>() {
            return Some(element.id());
        }
    }
    None
}

// [version - 1][ec level]
const BINARY_SIZE_TABLE: [[u16; 4]; 40] = [
    [17, 14, 11, 7],
    [32, 26, 20, 14],
    [53, 42, 32, 24],
    [78, 62, 46, 34],
    [106, 84, 60, 44],
    [134, 106, 74, 58],
    [154, 122, 86, 64],
    [192, 152, 108, 84],
    [230, 180, 130, 98],
    [271, 213, 151, 119],
    [321, 251, 177, 137],
    [367, 287, 203, 155],
    [425, 311, 241, 177],
    [458, 362, 258, 194],
    [520, 412, 292, 220],
    [586, 450, 322, 250],
    [644, 504, 364, 280],
    [718, 560, 394, 310],
    [792, 624, 442, 338],
    [858, 666, 482, 382],
    [929, 711, 509, 403],
    [1003, 779, 565, 439],
    [1091, 857, 611, 461],
    [1171, 911, 661, 511],
    [1273, 997, 715, 535],
    [1367, 1059, 751, 593],
    [1465, 1125, 805, 625],
    [1528, 1190, 868, 658],
    [1628, 1264, 908, 698],
    [1732, 1370, 982, 742],
    [1840, 1452, 1030, 790],
    [1952, 1538, 1112, 842],
    [2068, 1628, 1168, 898],
    [2188, 1722, 1228, 958],
    [2303, 1809, 1283, 983],
    [2431, 1911, 1351, 1051],
    [2563, 1989, 1423, 1093],
    [2699, 2099, 1499, 1139],
    [2809, 2213, 1579, 1219],
    [2953 - 2, 2331 - 2, 1663 - 2, 1273 - 2],
];

#[derive(Serialize)]
pub struct Context2DAttributes {
    alpha: bool,
}
