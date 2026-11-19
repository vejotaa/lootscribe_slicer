pub struct PricingResult {
    pub custo: f64,        // Custo total de produção (já aplicando fail_rate)
    pub lucro_val: f64,     // Lucro líquido (valor_base - custo)
    pub shopee_val: f64,    // Valor retido pela Shopee (comissão)
    pub margem: f64,       // Margem real (%)
    pub valor_venda: f64,   // Preço de venda final sugerido
}

/// Calcula custos e preço ideal para loja Shopee, com fail_rate aplicado ao custo
pub fn calculate_shopee_price(
    printing_time: f64,        // horas
    material_usage: f64,       // gramas
    material_price: f64,       // R$/kg
    profit: f64,               // multiplicador, ex: 2.0 = +100%
    fail_rate: f64,            // 1.0 = 0% falha, 1.1 = +10%
    paint: f64,                // custo extra de pintura, R$
    time_cost: f64,            // custo por hora de impressão, R$/h
) -> PricingResult {
    // 1. Custo base de produção
    let nozzle_cost = (material_usage * 25.0) / 5000.0;
    let material_cost = material_usage * (material_price / 1000.0);
    let machine_cost = printing_time * time_cost;
    let custo_bruto = machine_cost + nozzle_cost + material_cost + paint;

    // Aplica taxa de falha no custo de produção (risco/desperdício)
    let custo = custo_bruto * fail_rate;

    // 2. Base de lucro sobre o custo com falha
    let valor_base = custo * profit;

    // 3. Regras de comissão Shopee
    let (taxa_fixa, taxa_porc) = if valor_base < 79.99 {
        (4.0, 1.20)
    } else if valor_base < 99.99 {
        (16.0, 1.14)
    } else {
        (20.0, 1.14)
    };

    let valor_venda = valor_base * taxa_porc + taxa_fixa;
    let shopee_cut = valor_venda - valor_base;
    let lucro_val = valor_base - custo;
    let margem = if valor_venda > 0.0 {
        ((valor_venda - custo) / valor_venda) * 100.0
    } else {
        0.0
    };

    PricingResult {
        custo,
        lucro_val,
        shopee_val: shopee_cut,
        margem,
        valor_venda,
    }
}

pub fn get_input(prompt: &str, default_val: &str) -> String {
    use std::io::{self, Write};
    print!("{} [padrão: {}]: ", prompt, default_val);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Falha ao ler entrada");
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default_val.to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn interactive_input() -> (f64, f64, f64, f64, f64, f64, f64) {
    let printing_time = get_input("Tempo de Impressão (horas)", "5").parse().unwrap_or(5.0);
    let material_usage = get_input("Material Usado (gramas)", "120").parse().unwrap_or(120.0);
    let material_price = get_input("Preço do kg do Filamento (R$)", "125").parse().unwrap_or(125.0);
    let time_cost = get_input("Custo Hora Máquina (R$)", "0.8").parse().unwrap_or(0.80);
    let paint = get_input("Custo Pintura/Extra (R$)", "0").parse().unwrap_or(0.0);
    let profit = get_input("Multiplicador Lucro (ex: 2.0)", "2.0").parse().unwrap_or(2.0);
    let fail_rate = get_input("Taxa de Falha (ex: 1.1)", "1.1").parse().unwrap_or(1.1);
    (printing_time, material_usage, material_price, time_cost, paint, profit, fail_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shopee_tier_under_79() {
        // custo_bruto = 0.5 + 0.1 + 2.0 + 0 = 2.60
        // fail_rate = 1.0 -> custo = 2.60
        // valor_base = 2.60 * 2.0 = 5.20 (< 79.99)
        // valor_venda = 5.20 * 1.20 + 4.0 = 6.24 + 4.0 = 10.24
        let res = calculate_shopee_price(1.0, 20.0, 100.0, 2.0, 1.0, 0.0, 0.5);
        assert!((res.valor_venda - 10.24).abs() < 0.01);
        // lucro_val = valor_base - custo = 5.20 - 2.60 = 2.60
        assert!((res.lucro_val - 2.60).abs() < 0.01);
    }

    #[test]
    fn test_shopee_tier_mid_79_99() {
        // custo_bruto = 12.0 + 1.1 + 27.5 + 0 = 40.6
        // fail_rate = 1.0 -> custo = 40.6
        // valor_base = 40.6 * 2.0 = 81.2 (79.99..99.99)
        // valor_venda = 81.2 * 1.14 + 16.0 = 92.568 + 16.0 = 108.568
        let res = calculate_shopee_price(12.0, 220.0, 125.0, 2.0, 1.0, 0.0, 1.0);
        assert!((res.valor_venda - 108.57).abs() < 0.1);
        // lucro_val = 81.2 - 40.6 = 40.6
        assert!((res.lucro_val - 40.6).abs() < 0.01);
    }

    #[test]
    fn test_shopee_tier_above_99() {
        // Cenário que deve cair no tier >= 99.99
        // Aumentar valor_base forçando tier alto
        let res = calculate_shopee_price(30.0, 500.0, 200.0, 3.0, 1.1, 15.0, 2.0);
        // margem deve ser positiva se valor_venda > 0
        assert!(res.margem > 0.0);
        // custo deve ser custo_bruto * fail_rate
        let custo_bruto = 30.0 * 2.0 + (500.0 * 25.0 / 5000.0) + (500.0 * 200.0 / 1000.0) + 15.0;
        let expected_custo = custo_bruto * 1.1;
        assert!((res.custo - expected_custo).abs() < 0.1);
    }
}