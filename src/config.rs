use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub config_dir: PathBuf,
    pub material_price: f64,
    pub time_cost: f64,
    pub profit: f64,
    pub fail_rate: f64,
    pub paint: f64,
}

impl Config {
    pub fn from_cli_args(
        config_dir: Option<String>,
        material_price: Option<f64>,
        time_cost: Option<f64>,
        profit: Option<f64>,
        fail_rate: Option<f64>,
        paint: Option<f64>,
    ) -> Self {
        let default_config_dir = config_dir
            .map(PathBuf::from)
            .or_else(|| std::env::var("ORCA_CONFIG_DIR").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("k1c_profile"));

        let material_price = material_price.unwrap_or(125.0);
        let time_cost = time_cost.unwrap_or(0.80);
        let profit = profit.unwrap_or(2.0);
        let fail_rate = fail_rate.unwrap_or(1.1);
        let paint = paint.unwrap_or(0.0);

        Self {
            config_dir: default_config_dir,
            material_price,
            time_cost,
            profit,
            fail_rate,
            paint,
        }
    }

    pub fn k1c_profile_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn material_price(&self) -> f64 {
        self.material_price
    }

    pub fn time_cost(&self) -> f64 {
        self.time_cost
    }

    pub fn profit(&self) -> f64 {
        self.profit
    }

    pub fn fail_rate(&self) -> f64 {
        self.fail_rate
    }

    pub fn paint(&self) -> f64 {
        self.paint
    }
}