use crate::common::*;
use crate::traits::service_traits::chart_service::*;
use plotters::prelude::*;

#[derive(Debug, Clone, new)]
pub struct ChartServiceImpl;

impl ChartServiceImpl {
    #[doc = "Helper function to determine Y-axis range with padding"]
    fn calculate_y_range(&self, values: &[i64]) -> (i64, i64) {
        if values.is_empty() {
            return (0, 100);
        }

        let min_val: i64 = *values.iter().min().unwrap_or(&0);
        let max_val: i64 = *values.iter().max().unwrap_or(&100);

        let padding: i64 = ((max_val - min_val) as f64 * 0.1).max(1.0) as i64;

        let y_min: i64 = (min_val - padding).max(0);
        let y_max: i64 = max_val + padding;

        (y_min, y_max)
    }
}

#[async_trait]
impl ChartService for ChartServiceImpl {
    async fn generate_line_chart(
        &self,
        title: &str,
        x_labels: Vec<String>,
        y_data: Vec<i64>,
        output_path: &std::path::Path,
        x_label: &str,
        y_label: &str,
    ) -> anyhow::Result<()> {
        if x_labels.len() != y_data.len() {
            return Err(anyhow!(
                "[ChartServiceImpl->generate_line_chart] X labels and Y data must have the same length: {} vs {}",
                x_labels.len(),
                y_data.len()
            ));
        }

        if x_labels.is_empty() {
            return Err(anyhow!(
                "[ChartServiceImpl->generate_line_chart] Cannot generate chart with empty data"
            ));
        }

        /* Create parent directory if it doesn't exist */
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let output_path_str: String = output_path.to_string_lossy().to_string();
        let title: String = title.to_string();
        let x_label: String = x_label.to_string();
        let y_label: String = y_label.to_string();

        /* Calculate y_range before moving into closure */
        let (y_min, y_max) = self.calculate_y_range(&y_data);

        let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> =
            tokio::task::spawn_blocking(move || {
                /* ---- 여기부터는 동기 코드 (plotters) ---- */
                let root = BitMapBackend::new(&output_path_str, (1400, 700)).into_drawing_area();
                root.fill(&RGBColor(20, 20, 20))?;

                let mut chart = ChartBuilder::on(&root)
                    .caption(
                        &title,
                        ("sans-serif", 40)
                            .into_font()
                            .color(&RGBColor(240, 240, 240)),
                    )
                    .margin(30)
                    .x_label_area_size(70)
                    .y_label_area_size(90)
                    .build_cartesian_2d(0..x_labels.len() - 1, y_min..y_max)?;

                let line_color: RGBColor = RGBColor(0, 191, 255);
                let grid_color: RGBColor = RGBColor(60, 60, 60);
                let text_color: RGBColor = RGBColor(200, 200, 200);

                chart
                    .configure_mesh()
                    .x_desc(&x_label)
                    .y_desc(&y_label)
                    .x_labels(x_labels.len().min(10))
                    .y_labels(10)
                    .axis_style(ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(2))
                    .light_line_style(ShapeStyle::from(&grid_color).stroke_width(1))
                    .bold_line_style(ShapeStyle::from(&grid_color).stroke_width(2))
                    .x_label_style(("sans-serif", 18).into_font().color(&text_color))
                    .y_label_style(("sans-serif", 30).into_font().color(&text_color))
                    .x_label_formatter(&|x| {
                        if *x < x_labels.len() {
                            x_labels[*x].clone()
                        } else {
                            String::new()
                        }
                    })
                    .y_label_formatter(&|y| {
                        /* Inline format_number logic to avoid Self in closure */
                        let s: String = y.to_string();
                        let mut result: String = String::new();
                        let mut count: i32 = 0;
                        for c in s.chars().rev() {
                            if count == 3 {
                                result.push(',');
                                count = 0;
                            }
                            result.push(c);
                            count += 1;
                        }
                        result.chars().rev().collect()
                    })
                    .draw()?;

                chart.draw_series(LineSeries::new(
                    y_data.iter().enumerate().map(|(i, &y)| (i, y)),
                    ShapeStyle::from(&line_color).stroke_width(3),
                ))?;

                root.present()?;
                Ok(())
            });

        let drawing_result: Result<(), anyhow::Error> = handle.await.context(
            "[ChartServiceImpl->generate_line_chart] blocking task join failed (panic/cancelled)",
        )?;

        drawing_result.context("[ChartServiceImpl->generate_line_chart] drawing/present failed")?;

        info!("Line chart generated successfully: {:?}", output_path);

        Ok(())
    }
}
