use bearings_shared::models::Event;
use leptos::*;

#[server(GetNowEvents, "/api")]
pub async fn get_now_events() -> Result<Vec<Event>, ServerFnError> {
    // Wire up to SupabaseClient via use_context in Phase 3
    Ok(vec![])
}

#[component]
pub fn NowPage() -> impl IntoView {
    let events = create_resource(|| (), |_| async { get_now_events().await });

    view! {
        <section>
            <h1 style="font-size:18px;font-weight:700;color:#5C3D2E;margin-bottom:4px">
                "Now"
            </h1>
            <p style="font-size:12px;color:#777;margin-bottom:12px">
                "What the bear world is doing in the next 30 days."
            </p>

            <Suspense fallback=|| view! {
                <div style="text-align:center;padding:24px;color:#777">"Loading events\u{2026}"</div>
            }>
                {move || events.get().map(|result| match result {
                    Ok(evs) if evs.is_empty() => view! {
                        <div style="text-align:center;padding:24px;color:#777">
                            <div style="font-size:32px;margin-bottom:8px">"\u{1F43B}"</div>
                            <div style="font-size:13px;font-weight:600">
                                "Nothing in the next 30 days"
                            </div>
                            <a href="/coming-up"
                               style="font-size:12px;color:#D4860A;margin-top:4px;display:block">
                                "Browse upcoming events \u{2192}"
                            </a>
                        </div>
                    }.into_view(),
                    Ok(evs) => evs.iter().map(|ev| view! {
                        <EventCard event=ev.clone()/>
                    }).collect_view(),
                    Err(_) => view! {
                        <div style="color:#C0392B;padding:12px">"Failed to load events."</div>
                    }.into_view(),
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn EventCard(event: Event) -> impl IntoView {
    let city_country = match (&event.city, &event.country) {
        (Some(c), Some(co)) => format!("{c}, {co}"),
        (Some(c), None) => c.clone(),
        (None, Some(co)) => co.clone(),
        (None, None) => String::new(),
    };
    let name = event.name.clone();
    let link = event.link.clone();

    view! {
        <div style="border:1px solid #EDE0D4;border-radius:10px;padding:14px;margin-bottom:10px">
            <div style="font-weight:600;font-size:14px">{name}</div>
            <div style="font-size:12px;color:#777;margin-top:2px">{city_country}</div>
            {link.map(|url| view! {
                <a href={url} target="_blank" rel="noopener"
                   style="display:inline-block;margin-top:8px;font-size:12px;
                          color:#D4860A;font-weight:600">
                    "Info \u{2192}"
                </a>
            })}
        </div>
    }
}
