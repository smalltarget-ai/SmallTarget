#[cfg(test)]
mod tests {
    use image::GenericImageView;
    use vision::monitor::list_monitors;

    #[tokio::test]
    async fn test_multi_monitor_capture() {
        let monitors = list_monitors().await;
        // create output directory
        let output_dir = "target/debug/tests/capture_screen_test";
        std::fs::create_dir_all(output_dir).unwrap();
        for monitor in monitors {
            let image = monitor.capture_image().await.expect(&format!(
                "Failed to capture monitor {} ({:?})",
                monitor.id(),
                monitor.dimensions()
            ));
            println!("monitor {} screenshot {:?}",  monitor.id(),image.dimensions());
            // save image to file
            let path = format!("{}/image_{}.png", output_dir, monitor.id());
            image.save(path).unwrap();
        }
    }
}