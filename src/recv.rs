use yew::prelude::*;
use yew_router::prelude::*;

pub struct RecvPage;

impl Component for RecvPage {
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
            <b>{ "SEND PAGE" }</b>
        }
    }
}
