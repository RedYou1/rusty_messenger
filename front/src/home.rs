use dioxus::prelude::Scope;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::room::OpRoomId;
use crate::AccountManager;
use crate::Rooms;
use crate::Route;

#[inline_props]
pub fn Home(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();

    let nav = use_navigator(cx);

    match user.read().user() {
        Some(_) => {
            if let Some(room) = rooms.read().0.keys().last() {
                nav.replace(Route::Conv { room_id: *room });
            } else {
                nav.replace(Route::SideBar {
                    room_id: OpRoomId::new_empty(),
                });
            }
        }
        None => {
            nav.replace(Route::LogIn {});
        }
    };

    render! {
        div{"Loading..."}
    }
}
