use crate::common::*;

#[async_trait]
pub trait ChartService: Send + Sync {
    #[doc = "
        Generate a line chart from time-series data and save it as an image file
        # Arguments
        * `title` - Chart title
        * `x_labels` - Labels for X-axis (e.g., timestamps or dates)
        * `y_data` - Data points for Y-axis
        * `output_path` - Path where the chart image will be saved
        * `x_label` - Label for X-axis
        * `y_label` - Label for Y-axis
    "]
    async fn generate_line_chart(
        &self,
        title: &str,
        x_labels: Vec<String>,
        y_data: Vec<i64>,
        output_path: &Path,
        x_label: &str,
        y_label: &str,
    ) -> anyhow::Result<()>;
}
