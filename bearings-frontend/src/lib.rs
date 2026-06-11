use leptos::*;
use leptos_meta::*;
use leptos_router::*;

pub mod components;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html lang="en"/>
        <Title formatter=|text| format!("{text} — Bearings")/>
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Bear events, places, and community — worldwide."/>

        <Router>
            <Shell>
                <Routes>
                    <Route path="/" view=components::now::NowPage/>
                    <Route path="/coming-up" view=components::coming_up::ComingUpPage/>
                </Routes>
            </Shell>
        </Router>
    }
}

#[component]
fn Shell(children: Children) -> impl IntoView {
    view! {
        <div style="max-width:480px;margin:0 auto;font-family:system-ui,sans-serif;padding:0 12px">
            <header style="padding:16px 0 8px;border-bottom:1px solid #EDE0D4">
                <div style="font-size:22px;font-weight:800;color:#5C3D2E;letter-spacing:-.5px">
                    "BEARINGS"
                </div>
                <Nav/>
            </header>
            <main>
                {children()}
            </main>
        </div>
    }
}

#[component]
fn Nav() -> impl IntoView {
    let zones = [
        ("/", "Now"),
        ("/coming-up", "Coming Up"),
        ("/places", "Places"),
        ("/titles", "Titles"),
        ("/clubs", "Clubs"),
    ];

    view! {
        <nav style="display:flex;flex-wrap:wrap;gap:4px;padding-top:8px">
            {zones.iter().map(|(path, label)| {
                let href = path.to_string();
                view! {
                    <a href={href}
                       style="font-size:12px;font-weight:600;padding:4px 10px;border-radius:20px;
                              border:1px solid #D4A574;color:#5C3D2E;text-decoration:none">
                        {*label}
                    </a>
                }
            }).collect_view()}
        </nav>
    }
}
