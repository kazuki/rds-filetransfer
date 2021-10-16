mod header;
mod home;
mod recv;
mod routes;
mod send;

use yew::prelude::*;
use yew_router::prelude::*;

use home::HomePage;
use recv::RecvPage;
use routes::{Anchor, AppRoute};
use send::SendPage;

#[derive(Debug)]
pub struct App;

impl Component for App {
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
            <>
            <div class="navbar">
                <Anchor route=AppRoute::Home>{ "HOME" }</Anchor>
                <Anchor route=AppRoute::Send>{ "送信" }</Anchor>
                <Anchor route=AppRoute::Receive>{ "受信" }</Anchor>
            </div>
            <Router<AppRoute, ()>
                render = Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Home => html!{ <HomePage /> },
                        AppRoute::Send => html!{ <SendPage /> },
                        AppRoute::Receive => html!{ <RecvPage /> },
                    }
                })
                />
                </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
