use dioxus::prelude::Scope;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::room::OpRoomId;
use crate::AccountManager;
use crate::Rooms;
use crate::Route;

#[inline_props]
pub fn Home(cx: Scope) -> Element {
    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();

    let navigator = use_navigator(cx);

    match account_manager.read().current_user() {
        Some(_) => {
            if let Some(room) = rooms.read().0.keys().last() {
                navigator.replace(Route::Conv { room_id: *room });
            } else {
                navigator.replace(Route::SideBar {
                    room_id: OpRoomId::new_empty(),
                });
            }
        }
        None => {
            navigator.replace(Route::LogIn {});
        }
    };

    render! {
        div{"Loading..."}
    }
}
