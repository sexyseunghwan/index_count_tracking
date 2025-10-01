use crate::common::*;
use crate::traits::service_traits::chart_service::*;
use plotters::prelude::*;
use std::path::PathBuf;

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

    #[doc = "Helper function to format large numbers with comma separators"]
    fn format_number(n: i64) -> String {
        let s: String = n.to_string();
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
    }
}

#[async_trait]
impl ChartService for ChartServiceImpl {
    
    #[doc = ""]
    async fn generate_line_chart(
        &self,
        title: &str,
        x_labels: Vec<String>,
        y_data: Vec<i64>,
        output_path: &PathBuf,
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
            return Err(anyhow!("[ChartServiceImpl->generate_line_chart] Cannot generate chart with empty data"));
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

        let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> = tokio::task::spawn_blocking(move || {
            /* ---- 여기부터는 동기 코드 (plotters) ---- */ 
            let root = BitMapBackend::new(&output_path_str, (1400, 700)).into_drawing_area();
            root.fill(&RGBColor(20, 20, 20))?;

            let mut chart = ChartBuilder::on(&root)
                .caption(&title, ("sans-serif", 40).into_font().color(&RGBColor(240, 240, 240)))
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
                .label_style(("sans-serif", 16).into_font().color(&text_color))
                .x_label_formatter(&|x| {
                    if *x < x_labels.len() { x_labels[*x].clone() } else { String::new() }
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


        // 5) 에러를 단계별로 분리해 가독성 있게 처리
        let drawing_result: Result<(), anyhow::Error> = handle
            .await
            .context("[ChartServiceImpl->generate_line_chart] blocking task join failed (panic/cancelled)")?;

        drawing_result
            .context("[ChartServiceImpl->generate_line_chart] drawing/present failed")?;

        info!("Line chart generated successfully: {:?}", output_path);
        
        Ok(())
    }

    // async fn generate_multi_line_chart(
    //     &self,
    //     title: &str,
    //     x_labels: Vec<String>,
    //     series_data: Vec<(String, Vec<i64>)>,
    //     output_path: &PathBuf,
    //     x_label: &str,
    //     y_label: &str,
    // ) -> anyhow::Result<()> {
    //     if series_data.is_empty() {
    //         return Err(anyhow!("Cannot generate chart with empty series data"));
    //     }

    //     // Validate all series have the same length as x_labels
    //     for (name, data) in &series_data {
    //         if data.len() != x_labels.len() {
    //             return Err(anyhow!(
    //                 "Series '{}' has {} data points but x_labels has {} points",
    //                 name,
    //                 data.len(),
    //                 x_labels.len()
    //             ));
    //         }
    //     }

    //     if x_labels.is_empty() {
    //         return Err(anyhow!("Cannot generate chart with empty data"));
    //     }

    //     // Create parent directory if it doesn't exist
    //     if let Some(parent) = output_path.parent() {
    //         tokio::fs::create_dir_all(parent).await?;
    //     }

    //     let output_path_str = output_path.to_string_lossy().to_string();
    //     let title = title.to_string();
    //     let x_label = x_label.to_string();
    //     let y_label = y_label.to_string();

    //     // Run blocking operation in a separate thread
    //     tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
    //         let root = BitMapBackend::new(&output_path_str, (1200, 600)).into_drawing_area();
    //         root.fill(&WHITE)?;

    //         // Calculate Y range from all series
    //         let all_values: Vec<i64> = series_data
    //             .iter()
    //             .flat_map(|(_, data)| data.iter().copied())
    //             .collect();
    //         let (y_min, y_max) = Self::calculate_y_range(&all_values);

    //         let mut chart = ChartBuilder::on(&root)
    //             .caption(&title, ("sans-serif", 30).into_font())
    //             .margin(20)
    //             .x_label_area_size(60)
    //             .y_label_area_size(80)
    //             .build_cartesian_2d(0..x_labels.len() - 1, y_min..y_max)?;

    //         chart
    //             .configure_mesh()
    //             .x_desc(&x_label)
    //             .y_desc(&y_label)
    //             .x_labels(x_labels.len().min(10))
    //             .y_labels(10)
    //             .x_label_formatter(&|x| {
    //                 if *x < x_labels.len() {
    //                     x_labels[*x].clone()
    //                 } else {
    //                     String::new()
    //                 }
    //             })
    //             .y_label_formatter(&|y| Self::format_number(*y))
    //             .draw()?;

    //         // Define colors for different series
    //         let colors = vec![
    //             &BLUE,
    //             &RED,
    //             &GREEN,
    //             &MAGENTA,
    //             &CYAN,
    //             &BLACK,
    //             &RGBColor(255, 165, 0), // Orange
    //             &RGBColor(128, 0, 128), // Purple
    //         ];

    //         // Draw each series
    //         for (idx, (series_name, y_data)) in series_data.iter().enumerate() {
    //             let color = colors[idx % colors.len()];

    //             // Draw line
    //             chart
    //                 .draw_series(LineSeries::new(
    //                     y_data.iter().enumerate().map(|(i, &y)| (i, y)),
    //                     color,
    //                 ))?
    //                 .label(series_name)
    //                 .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

    //             // Draw points
    //             chart.draw_series(PointSeries::of_element(
    //                 y_data.iter().enumerate().map(|(i, &y)| (i, y)),
    //                 5,
    //                 color,
    //                 &|c, s, st| {
    //                     return EmptyElement::at(c) + Circle::new((0, 0), s, st.filled());
    //                 },
    //             ))?;
    //         }

    //         // Configure legend
    //         chart
    //             .configure_series_labels()
    //             .background_style(&WHITE.mix(0.8))
    //             .border_style(&BLACK)
    //             .draw()?;

    //         root.present()?;
    //         Ok(())
    //     })
    //     .await??;

    //     info!("Multi-line chart generated successfully: {:?}", output_path);
    //     Ok(())
    // }

    // async fn generate_bar_chart(
    //     &self,
    //     title: &str,
    //     categories: Vec<String>,
    //     values: Vec<i64>,
    //     output_path: &PathBuf,
    //     x_label: &str,
    //     y_label: &str,
    // ) -> anyhow::Result<()> {
    //     if categories.len() != values.len() {
    //         return Err(anyhow!(
    //             "Categories and values must have the same length: {} vs {}",
    //             categories.len(),
    //             values.len()
    //         ));
    //     }

    //     if categories.is_empty() {
    //         return Err(anyhow!("Cannot generate chart with empty data"));
    //     }

    //     // Create parent directory if it doesn't exist
    //     if let Some(parent) = output_path.parent() {
    //         tokio::fs::create_dir_all(parent).await?;
    //     }

    //     let output_path_str = output_path.to_string_lossy().to_string();
    //     let title = title.to_string();
    //     let x_label = x_label.to_string();
    //     let y_label = y_label.to_string();

    //     // Run blocking operation in a separate thread
    //     tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
    //         let root = BitMapBackend::new(&output_path_str, (1200, 600)).into_drawing_area();
    //         root.fill(&WHITE)?;

    //         let (y_min, y_max) = Self::calculate_y_range(&values);

    //         let mut chart = ChartBuilder::on(&root)
    //             .caption(&title, ("sans-serif", 30).into_font())
    //             .margin(20)
    //             .x_label_area_size(60)
    //             .y_label_area_size(80)
    //             .build_cartesian_2d((0..categories.len()).into_segmented(), y_min..y_max)?;

    //         chart
    //             .configure_mesh()
    //             .x_desc(&x_label)
    //             .y_desc(&y_label)
    //             .y_labels(10)
    //             .x_label_formatter(&|x| match x {
    //                 SegmentValue::CenterOf(idx) => {
    //                     if *idx < categories.len() {
    //                         categories[*idx].clone()
    //                     } else {
    //                         String::new()
    //                     }
    //                 }
    //                 _ => String::new(),
    //             })
    //             .y_label_formatter(&|y| Self::format_number(*y))
    //             .draw()?;

    //         // Draw bars
    //         chart.draw_series(
    //             Histogram::vertical(&chart)
    //                 .style(BLUE.filled())
    //                 .data(values.iter().enumerate().map(|(i, &v)| (i, v))),
    //         )?;

    //         root.present()?;
    //         Ok(())
    //     })
    //     .await??;

    //     info!("Bar chart generated successfully: {:?}", output_path);
    //     Ok(())
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::path::PathBuf;

//     #[tokio::test]
//     async fn test_generate_line_chart() {
//         let chart_service = ChartServiceImpl::new();

//         let x_labels = vec![
//             "2024-01".to_string(),
//             "2024-02".to_string(),
//             "2024-03".to_string(),
//             "2024-04".to_string(),
//             "2024-05".to_string(),
//         ];
//         let y_data = vec![1000, 1500, 1200, 1800, 2000];
//         let output_path = PathBuf::from("test_output/line_chart.png");

//         let result = chart_service
//             .generate_line_chart(
//                 "Monthly Document Count",
//                 x_labels,
//                 y_data,
//                 &output_path,
//                 "Month",
//                 "Count",
//             )
//             .await;

//         assert!(result.is_ok());
//         assert!(output_path.exists());
//     }

//     #[tokio::test]
//     async fn test_generate_multi_line_chart() {
//         let chart_service = ChartServiceImpl::new();

//         let x_labels = vec![
//             "Jan".to_string(),
//             "Feb".to_string(),
//             "Mar".to_string(),
//             "Apr".to_string(),
//         ];
//         let series_data = vec![
//             ("Index A".to_string(), vec![1000, 1500, 1200, 1800]),
//             ("Index B".to_string(), vec![800, 900, 1100, 1300]),
//             ("Index C".to_string(), vec![1200, 1400, 1600, 2000]),
//         ];
//         let output_path = PathBuf::from("test_output/multi_line_chart.png");

//         let result = chart_service
//             .generate_multi_line_chart(
//                 "Index Comparison",
//                 x_labels,
//                 series_data,
//                 &output_path,
//                 "Month",
//                 "Document Count",
//             )
//             .await;

//         assert!(result.is_ok());
//         assert!(output_path.exists());
//     }

//     #[tokio::test]
//     async fn test_generate_bar_chart() {
//         let chart_service = ChartServiceImpl::new();

//         let categories = vec![
//             "Index A".to_string(),
//             "Index B".to_string(),
//             "Index C".to_string(),
//             "Index D".to_string(),
//         ];
//         let values = vec![1500, 2000, 1200, 1800];
//         let output_path = PathBuf::from("test_output/bar_chart.png");

//         let result = chart_service
//             .generate_bar_chart(
//                 "Document Count by Index",
//                 categories,
//                 values,
//                 &output_path,
//                 "Index Name",
//                 "Document Count",
//             )
//             .await;

//         assert!(result.is_ok());
//         assert!(output_path.exists());
//     }
// }
