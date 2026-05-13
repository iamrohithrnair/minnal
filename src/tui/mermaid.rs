pub use minnal_tui_mermaid::*;

pub fn install_minnal_mermaid_hooks() {
    minnal_tui_mermaid::set_log_hooks(crate::logging::info, crate::logging::warn);
    minnal_tui_mermaid::set_render_completed_hook(|| {
        crate::bus::Bus::global().publish(crate::bus::BusEvent::MermaidRenderCompleted);
    });
    minnal_tui_mermaid::set_memory_snapshot_hook(|| {
        let snapshot = crate::process_memory::snapshot_with_source("client:mermaid:memory");
        minnal_tui_mermaid::ProcessMemorySnapshot {
            rss_bytes: snapshot.rss_bytes,
            peak_rss_bytes: snapshot.peak_rss_bytes,
            virtual_bytes: snapshot.virtual_bytes,
        }
    });
}
