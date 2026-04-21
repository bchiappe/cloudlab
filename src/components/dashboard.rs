use leptos::prelude::*;
use crate::components::icons::{Cpu, Memory, HardDrive, CheckCircle, Desktop, Cube, SquaresFour};

#[component]
pub fn DashboardOverview() -> impl IntoView {
    view! {
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-100 flex items-center gap-3">
                <SquaresFour class="w-8 h-8 text-blue-500" />
                "Cluster Overview"
            </h1>
            <p class="text-gray-400 mt-2">"Health status and resource utilization across all federated nodes."</p>
        </div>

        // High-level Stats Cards
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard title="Healthy Hosts" count="12" total="12" icon=view!{<Desktop/>}.into_any() status="healthy" />
            <StatCard title="Running VMs" count="148" total="156" icon=view!{<Cube/>}.into_any() status="healthy" />
            <StatCard title="Storage Alerts" count="2" total="8" icon=view!{<HardDrive/>}.into_any() status="warning" />
            <div class="p-6 bg-gray-900 border border-gray-800 rounded-xl shadow-lg flex items-center justify-between">
                <div>
                    <h3 class="text-gray-400 text-sm font-medium mb-1">"Cluster Status"</h3>
                    <div class="flex items-center gap-2">
                        <CheckCircle class="w-6 h-6 text-green-500" />
                        <span class="text-xl font-bold text-gray-100">"Optimal"</span>
                    </div>
                </div>
            </div>
        </div>

        // Resource Utilization
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <ResourceGauge title="CPU Usage" percentage=42.0 unit="GHz" used="112" total="266" icon=view!{<Cpu/>}.into_any() color="bg-blue-500" />
            <ResourceGauge title="Memory Utilization" percentage=78.5 unit="TB" used="4.2" total="5.3" icon=view!{<Memory/>}.into_any() color="bg-purple-500" />
            <ResourceGauge title="Datastore Capacity" percentage=88.2 unit="TB" used="104.5" total="118.4" icon=view!{<HardDrive/>}.into_any() color="bg-orange-500" />
        </div>
    }
}



#[component]
fn StatCard(
    title: &'static str, 
    count: &'static str, 
    total: &'static str, 
    icon: AnyView, 
    status: &'static str
) -> impl IntoView {
    let status_color = match status {
        "healthy" => "text-green-500",
        "warning" => "text-orange-500",
        "critical" => "text-red-500",
        _ => "text-blue-500"
    };

    view! {
        <div class="p-6 bg-gray-900 border border-gray-800 rounded-xl shadow-lg flex flex-col justify-between group hover:border-gray-700 transition">
            <div class="flex justify-between items-start mb-4">
                <div class="p-2 bg-gray-800 rounded-lg text-gray-300 group-hover:text-white transition">
                    <div class="w-6 h-6">{icon}</div>
                </div>
                <span class=format!("text-xs font-semibold px-2 py-1 bg-gray-950 rounded-full border border-gray-800 {}", status_color)>
                    {if status == "healthy" { "Online" } else { "Review" }}
                </span>
            </div>
            <div>
                <h3 class="text-gray-400 text-sm font-medium mb-1">{title}</h3>
                <div class="flex items-baseline gap-2">
                    <span class="text-3xl font-bold text-gray-100">{count}</span>
                    <span class="text-sm font-medium text-gray-500">"/ " {total}</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ResourceGauge(
    title: &'static str,
    percentage: f32,
    unit: &'static str,
    used: &'static str,
    total: &'static str,
    icon: AnyView,
    color: &'static str
) -> impl IntoView {
    view! {
        <div class="p-6 bg-gray-900 border border-gray-800 rounded-xl shadow-lg flex flex-col">
            <div class="flex justify-between items-center mb-6">
                <div class="flex items-center gap-2 text-gray-200">
                    <div class="w-5 h-5 text-gray-400">{icon}</div>
                    <h3 class="font-semibold text-lg">{title}</h3>
                </div>
                <span class="text-xl font-bold tracking-tight text-white">{percentage}"%"</span>
            </div>
            
            <div class="w-full bg-gray-800 h-3 rounded-full mb-4 overflow-hidden shadow-inner">
                <div class=format!("h-full {} rounded-full transition-all duration-1000", color) style=format!("width: {}%", percentage)></div>
            </div>
            
            <div class="flex justify-between items-center text-sm font-medium text-gray-400 mt-auto">
                <span>"Used: " <span class="text-gray-200">{used} " " {unit}</span></span>
                <span>"Total: " {total} " " {unit}</span>
            </div>
        </div>
    }
}
