#[cfg(test)]
mod tests {
    use image::GenericImageView;
    use vision::capture_screenshot_by_window::{
        capture_all_visible_windows, CapturedWindow, WindowFilters,
    };
    use vision::monitor::{get_default_monitor, list_monitors};

    #[tokio::test]
    async fn test_multi_monitor_capture() {
        // create output directory
        let output_dir = "target/debug/tests/multi_monitor_capture";
        std::fs::create_dir_all(output_dir).unwrap();

        let monitors = list_monitors().await;
        for monitor in monitors {
            let image = monitor.capture_image().await.expect(&format!(
                "Failed to capture monitor {} ({:?})",
                monitor.id(),
                monitor.dimensions()
            ));
            // save image to file
            let path = format!("{}/image_{}.png", output_dir, monitor.id());
            image.save(path).unwrap();
            println!(
                "monitor {} screenshot {:?}",
                monitor.id(),
                image.dimensions()
            );
        }
    }

    #[tokio::test]
    async fn test_capture_window() {
        let output_dir = "target/debug/tests/capture_window";
        std::fs::create_dir_all(output_dir).unwrap();
        let monitors = list_monitors().await;
        for monitor in monitors {
            let windows =
                capture_all_visible_windows(&monitor, &WindowFilters::new(&[], &[]), true)
                    .await
                    .expect("Failed to capture windows");
            for window in windows {
                let image = window.image;
                let path = format!("{}/image_{}.png", output_dir, window.app_name);
                image.save(path).unwrap();
                println!(
                    "monitor {} window {} screenshot {:?}",
                    monitor.id(),
                    window.app_name,
                    image.dimensions()
                );
            }
        }
    }
}
