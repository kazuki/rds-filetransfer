extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod home;
mod routes;
mod send;
mod recv;
mod header;

use yew::prelude::*;
use yew_router::prelude::*;

use routes::{Anchor, AppRoute};
use home::HomePage;
use send::SendPage;
use recv::RecvPage;

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
