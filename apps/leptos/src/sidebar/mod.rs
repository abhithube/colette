use feed_list::FeedList;
use leptos::prelude::*;
use leptos_router::components::{Outlet, A};

use crate::common::icons::{Clock, Cog, Home, Wrench};

mod feed_item;
mod feed_list;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <div class="flex">
            <nav class="flex flex-col w-[300px] h-screen px-2">
                <ul class="flex grow flex-col">
                    <li class="mb-4">
                        <ul class="menu">
                            <li>
                                <A href="/">
                                    <Home {..} class="size-5" />
                                    "Home"
                                </A>
                            </li>
                            <li>
                                <A href="/feeds/manage">
                                    <Clock {..} class="size-5" />
                                    "Archived"
                                </A>
                            </li>
                            <li>
                                <A href="/feeds/manage">
                                    <Wrench {..} class="size-5" />
                                    "Manage Feeds"
                                </A>
                            </li>
                        </ul>
                    </li>
                    <li class="grow">
                        <ul class="menu">
                            <li class="menu-title">"Feeds"</li>
                            <FeedList />
                        </ul>
                    </li>
                    <li>
                        <ul class="menu">
                            <li>
                                <button>
                                    <Cog {..} class="size-5" />
                                    "Settings"
                                </button>
                            </li>
                        </ul>
                    </li>
                </ul>
            </nav>
            <div class="divider divider-horizontal w-0 mx-0" />
            <div class="grow">
                <Outlet />
            </div>
        </div>
    }
}
