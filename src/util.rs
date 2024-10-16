use indicatif::{ProgressBar, ProgressStyle};

/// Creates a progress bar for visual feedback during synchronization
pub fn create_progress_bar(total_steps: u64, info: String) -> ProgressBar {
    let pb = ProgressBar::new(total_steps);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{elapsed_precise}} {} {{bar:40.cyan/blue}} {{pos}}/{{len}} {{msg}}",
                info
            ))
            .unwrap()
            .progress_chars("##-"),
    );
    pb.tick();
    pb
}
