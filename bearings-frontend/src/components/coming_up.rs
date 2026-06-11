use leptos::*;

/// Coming Up zone — upcoming bear events beyond 30 days.
/// Placeholder until the data layer is wired via server functions.
#[component]
pub fn ComingUpPage() -> impl IntoView {
    view! {
        <section>
            <h1 style="font-size:18px;font-weight:700;color:#5C3D2E;margin-bottom:4px">
                "Coming Up"
            </h1>
            <p style="font-size:12px;color:#777;margin-bottom:12px">
                "Upcoming bear events worldwide."
            </p>
            <div style="text-align:center;padding:24px;color:#777">
                "Coming soon — data layer in progress."
            </div>
        </section>
    }
}
