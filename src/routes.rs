use yew_router::components::RouterAnchor;
use yew_router::prelude::Switch;

#[derive(Switch, Clone, Debug)]
pub enum AppRoute {
    #[to = "#/send"]
    Send,
    #[to = "#/receive"]
    Receive,
    #[to = "#/"]
    Home,
}

pub type Anchor = RouterAnchor<AppRoute>;
