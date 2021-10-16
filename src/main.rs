mod header;
mod home;
mod recv;
mod routes;
mod send;

use yew::prelude::*;
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;

use home::HomePage;
use recv::RecvPage;
use routes::{Anchor, AppRoute};
use send::SendPage;

pub struct App {
    #[allow(unused)]
    route_agent: Box<dyn Bridge<RouteAgent>>,
    current_route: Option<AppRoute>,
}

pub enum Msg {
    UpdateRoute(Route<()>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut route_agent = RouteAgent::bridge(link.callback(Msg::UpdateRoute));
        route_agent.send(RouteRequest::GetCurrentRoute);
        Self {
            route_agent,
            current_route: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateRoute(mut route) => {
                let r = route.route.as_str();
                if let Some(index) = r.find('#') {
                    route.route = r[index..].to_string();
                } else {
                    route.route = "#/".to_string();
                }
                self.current_route = AppRoute::switch(route);
            }
        }
        true
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
                {
                    if let Some(route) = &self.current_route {
                        match route {
                            AppRoute::Home => html!{ <HomePage /> },
                            AppRoute::Send => html!{ <SendPage /> },
                            AppRoute::Receive => html!{ <RecvPage /> },
                        }
                    } else {
                        html!{ { "not found" } }
                    }
                }
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
