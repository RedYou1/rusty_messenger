use dioxus::prelude::Scope;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::account::AccountManager;
use crate::Route;

#[inline_props]
pub fn Home(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();

    let nav = use_navigator(cx);

    match user.read().as_ref() {
        Some(_) => nav.replace(Route::Conv { room: 0 }),
        None => nav.replace(Route::LogIn {}),
    };

    render! {
        div{"?"}
    }
}
