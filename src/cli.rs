use clap::Parser;

/// Sliceia arquivos STL/3MF com o Orca Slicer (perfil K1C) e calcula o preço
/// de venda ideal para loja Shopee.
#[derive(Parser, Debug)]
#[command(name = "lootscribe-slicer", version, about, long_about = None)]
pub struct Args {
    /// Modo one-shot: processa um único arquivo e sai
    #[arg(long)]
    pub run: bool,

    /// Arquivo de entrada (.stl ou .3mf)
    #[arg(long, value_name = "CAMINHO")]
    pub file: Option<String>,

    /// Direório de configuração do Orca Slicer (machine/process/filament + system/*.opc)
    #[arg(long, value_name = "DIRETORIO")]
    pub config_dir: Option<String>,

    /// Desabilita o --orient 1 (auto-orientação)
    #[arg(long)]
    pub no_orient: bool,

    /// Desabilita o --arrange 1 (organizar na placa)
    #[arg(long)]
    pub no_arrange: bool,

    /// Preço do filamento, em R$/kg (padrão: 125)
    #[arg(long, value_name = "VALOR")]
    pub material_price: Option<f64>,

    /// Multiplicador de lucro, ex: 2.0 (padrão: 2.0)
    #[arg(long, value_name = "VALOR")]
    pub profit: Option<f64>,

    /// Taxa de falha, ex: 1.1 = +10% (padrão: 1.1)
    #[arg(long, value_name = "VALOR")]
    pub fail_rate: Option<f64>,

    /// Custo de pintura/extra, em R$ (padrão: 0)
    #[arg(long, value_name = "VALOR")]
    pub paint: Option<f64>,

    /// Custo por hora de impressão, em R$/h (padrão: 0.80)
    #[arg(long, value_name = "VALOR")]
    pub time_cost: Option<f64>,

    /// Não copiar resultados para a área de transferência
    #[arg(long)]
    pub no_clipboard: bool,
}