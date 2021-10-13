use yew::prelude::*;

pub struct HomePage;

impl Component for HomePage {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div style="padding-left: 1em">
                <p>
                    { "RDS等越しにファイルを転送するデモツールです。" }<br />
                    { "リモート側でQRコードを表示し、ローカル側はその画面をキャプチャすることでデータをファイル転送を実現します。" }
                </p>
                <p>
                    { "上部のナビゲーションメニューより、ファイルの送信または受信を選んでください。" }
                </p>
            </div>
        }
    }
}
