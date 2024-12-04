use colette_api::profile::{GetActiveResponse, Profile};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    StaticSegment,
};

use crate::{common::auth, login::LoginPage};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/colette.css" />

        <Title text="Welcome to Leptos" />

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <ParentRoute path=StaticSegment("") view=RootLayout>
                        <Route path=StaticSegment("/") view=HomePage />
                        <Route path=StaticSegment("/login") view=LoginPage />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn RootLayout() -> impl IntoView {
    let profile_res = Resource::new_blocking(
        || (),
        move |_| async move {
            auth::get_active_profile().await.ok().map(|e| match e {
                GetActiveResponse::Ok(profile) => profile,
            })
        },
    );

    let inner_view = Suspend::new(async move {
        let profile = profile_res.await;
        provide_context(RwSignal::new(profile));

        Outlet()
    });

    view! { <Suspense>{inner_view}</Suspense> }
}

#[component]
fn HomePage() -> impl IntoView {
    let profile = expect_context::<RwSignal<Option<Profile>>>();

    view! { <h1>{move|| profile.get().map(|e| e.title)}</h1> }
}
