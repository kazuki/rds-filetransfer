use yew_router::prelude::Switch;
use yew_router::components::RouterAnchor;

#[derive(Switch, Clone, Debug)]
pub enum AppRoute {
    #[to="/send"]
    Send,
    #[to="/receive"]
    Receive,
    #[to="/"]
    Home,
}

pub type Anchor = RouterAnchor<AppRoute>;
