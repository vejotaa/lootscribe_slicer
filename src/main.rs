use clap::Parser;
use anyhow::Context;

mod cli;
mod pricing;
mod slicer;
mod config;

fn main() -> anyhow::Result<()> {
    let cli_args = cli::Args::parse();
    let config = config::Config::from_cli_args(
        cli_args.config_dir,
        cli_args.material_price,
        cli_args.time_cost,
        cli_args.profit,
        cli_args.fail_rate,
        cli_args.paint,
    );

    if cli_args.run {
        let input_file = cli_args.file.context(
            "Arquivo de entrada (.stl ou .3mf) é necessário com --run",
        )?;

        let slice_res = slicer::slice_with_orca(
            &input_file,
            config.k1c_profile_dir().to_str().unwrap(),
            cli_args.no_orient,
            cli_args.no_arrange,
        )?;

        let price_res = pricing::calculate_shopee_price(
            slice_res.printing_time_hours,
            slice_res.material_usage_grams,
            config.material_price(),
            config.profit(),
            config.fail_rate(),
            config.paint(),
            config.time_cost(),
        );

        println!("=== Resultado da Slicing ===");
        println!("Arquivo: {}", input_file);
        println!("Tempo de impressão: {:.2} h", slice_res.printing_time_hours);
        println!("Material usado: {:.1} g", slice_res.material_usage_grams);
        println!("G-code gerado: {}", slice_res.gcode_path);
        println!("\n=== Cálculo Shopee ===");
        println!("Custo Total: R$ {:.2}", price_res.custo);
        println!("Lucro Líquido: R$ {:.2}", price_res.lucro_val);
        println!("Comissão Shopee: R$ {:.2}", price_res.shopee_val);
        println!("Margem: {:.1}%", price_res.margem);
        println!("Preço Sugerido: R$ {:.2}", price_res.valor_venda);

        if !cli_args.no_clipboard {
            let output_str = format!(
                "Preço Sugerido: R$ {:.2} | Custo: R$ {:.2} | Lucro: R$ {:.2} | Margem: {:.1}%",
                price_res.valor_venda, price_res.custo, price_res.lucro_val, price_res.margem
            );
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(output_str);
                println!("\n(Resultado copiado para a área de transferência)");
            }
        }
    } else {
        let (time, mat, price, t_cost, paint, prof, fail) = pricing::interactive_input();
        let price_res = pricing::calculate_shopee_price(time, mat, price, prof, fail, paint, t_cost);
        println!("\n=== Cálculo Shopee ===");
        println!("Custo Total: R$ {:.2}", price_res.custo);
        println!("Lucro Líquido: R$ {:.2}", price_res.lucro_val);
        println!("Comissão Shopee: R$ {:.2}", price_res.shopee_val);
        println!("Margem: {:.1}%", price_res.margem);
        println!("Preço Sugerido: R$ {:.2}", price_res.valor_venda);

        if !cli_args.no_clipboard {
            let output_str = format!(
                "Preço Sugerido: R$ {:.2} | Custo: R$ {:.2} | Lucro: R$ {:.2} | Margem: {:.1}%",
                price_res.valor_venda, price_res.custo, price_res.lucro_val, price_res.margem
            );
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(output_str);
                println!("\n(Resultado copiado para a área de transferência)");
            }
        }
    }

    Ok(())
}