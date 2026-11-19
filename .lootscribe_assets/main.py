Material_usage = None
Material_price = 125
Fail_rate = 1.1
Printing_time = None
Printing_timecost = 0.8
Lucro = None

Printing_time = int(input(f' Qual é o tempo de impressão?'))
Material_usage = int(input(f'Quanto do material se usa?'))

Nozzle_cost = ((Material_usage*25)/5000)
Material_cost = Material_usage*(Material_price/1000)

Pintura_cost = int(input((f'Qual o valor do pintura?')))

Custo = Printing_time*Printing_timecost+Nozzle_cost+Material_cost+Pintura_cost

print(Custo)

Lucro = float(input(f'Qual vai ser o lucro?'))

TaxaShopeePorcentagem = 1
TaxaShopeeFixa = 0

Valor_Base = int(Custo*Lucro)
Valor_Venda = Valor_Base*TaxaShopeePorcentagem+TaxaShopeeFixa

print(Valor_Venda)

if Valor_Venda < 79.99:
    TaxaShopeeFixa = 4
    TaxaShopeePorcentagem = 1.2
elif 79.99 < Valor_Venda < 99.99:
    TaxaShopeeFixa = 16
    TaxaShopeePorcentagem = 1.14
elif 99.99 < Valor_Venda < 199.99:
    TaxaShopeeFixa = 20
    TaxaShopeePorcentagem = 1.14

Valor_Venda = int((Valor_Venda*TaxaShopeePorcentagem+TaxaShopeeFixa)*Fail_rate)
Margem = 1-(Valor_Base/Valor_Venda)

print(f"O quando você ira cobrar, deve ser {Valor_Venda}, o que significa que o quanto eles roubam de você é {Valor_Venda-Valor_Base}. A margem real é {Margem:.2f}.")


