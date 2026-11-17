<h1 align="center">Fexcel</h1>
<p align="center"><img src="https://i.imgur.com/zuZuoek.jpeg" height=500></img></p>
<p align="center"><i>(fuck excel)</i></p>
<h2 align="center"> Um simples programa para controle de gastos </h2>

### Criado após tentativa falha de controlar os gastos usando o Excel. 
### Esse programa foca na simplicidade e rapidez mas sem abrir mão de tudo que um programa desse tipo precisa.
# Usando:
```bash
fexcel
```
Ao rodar o programa sem argumentos ele somente tem duas opções:
<p align="center"><img src="https://i.imgur.com/ukQu7MY.jpeg" height=100></img></p>
Também conhecidas como todas as que você precisa.

# Mas Fexcel foi feito com argumentos  práticos em mente:
## Filtro de início:
```bash
fexcel --ss 01/01/2026
```
Vai filtrar tudo de 01/01/2026 para frente

## Filtro de fim:
```bash
fexcel --to 31/01/2026
```
Vai filtrar tudo de 31/01/2026 para trás

## Filtro de N dias atrás
```bash
fexcel --t 30
```
Vai filtrar os últimos 30 dias

## Filtro Regex
```bash
fexcel --r "padaria"
```
Vai filtrar todas as entradas que correspondem a expressão regular "padaria" (por padrão letras minúsculas e maiúsculas são ignoradas)

## Todos de uma vez em qualquer combinação !
```bash
fexcel --r "padaria" --ss 01/01/2026 --to 31/01/2026
```
Vai filtrar todas as entradas que tenha "padaria" na descrição e estejam entre 01/01/2026 e 31/01/2026 (Você também pode usar "--t" aqui, mas ele ira sobrescrever o "--ss")

## Highlights
```bash
fexcel --hl
```
Mostra quando você bateu os recordes de quantidade de dinheiro
<p align="center"><img src="https://i.imgur.com/DoopyBc.jpeg" height=280></img></p>

## Entradas Saídas
```bash
fexcel --io
```
Mostra as entradas e saídas dividias por mês
<p align="center"><img src="https://i.imgur.com/cn52Cvz.jpeg" height=110></img></p>

# Configurações
As configurações são salvas na mesma pasta do executável e criadas automaticamente assim que ele roda pela primeira vez
```ini
history_file_path = "history.log"

#Parâmetros comentados de exemplo
#
#Objetivos, mostra a porcentagem que você já conseguiu desse objetivo
#
[objectives] 
Carro=10000 #E.G: Tenho 5000 no total, então aparecerá '❱ Carro: $10000 (50%)'
Celular=500

#Despesas, mostra a porcentagem que uma despesa impacta no total
#
[expenses]
Cartão de crédito=500 #E.G: Tenho 1000 no total, então aparecerá '❰ Cartão de crédito: $500 (50%)'

# Muda o estilo de print do programa
#
[style]
separator="=" #Muda o separador de linhas
separator_size=50 #Muda o tamanho do separador de linhas
trim_list_size=100 #Muda quantos registros serão ocultos ao ver os registro (eles só não aparecem, mas são somados e filtrados) (0 desativa)
trim_desc_size=50 #Muda o quantos caracteres são exibidos na descrição, se for mais do que definido é cortado e adicionado "..." (desabilitado por padrão)
```
(Observe que para o arquivo .ini ser lido corretamente os títulos das sessões devem estar presentes e descomentados)

# history.log
O arquivo no qual os registros são gravados, muito simples e pode ser editado com qualquer editor de texto caso necessário (o caminho e o nome podem ser modificado nas configurações)
```
01/01/2026:-10.50:Pão de queijo
02/01/2026:100:Venda
03/01/2026:-50:Janta fora
04/01/2026:-30:Mercado
...
```
(Observe que arquivo history.log deve sempre estar em order cronológica para os filtros funcionarem corretamente, algo que somente pode acontecer caso for editado externamente)
# Instruções para compilar:
Clonar esse repositório
```bash
git clone https://github.com/srtopster/fexcel.git
```
Entrar na pasta
```bash
cd fexcel
```
Construir com cargo
```bash
cargo build --release
```
O binário estará dentro de `/target/release`.

## Android
Eu uso esse programa 100% do tempo dentro do Termux, pois sempre estou com o meu celular por perto e assim realmente consigo registrar os meus gastos com praticidade.

Para criar o seu executável rodável no Termux você pode tanto baixar o Rust no Termux:
```bash
pkg install rust
```
E seguir os passos acima, ou fazer como eu faço e cross-compilar para evitar baixar muita coisa no Termux.<br>
Muito simples usando o [Cross](https://github.com/cross-rs/cross) no Linux (também funciona no Windows)
```bash
cross build --release --target armv7-linux-androideabi
```
Aí é só pegar o binário em `target/armv7-linux-androideabi/release` e enviar para o Termux.

[![brainmade.org](https://img.shields.io/badge/brainmade.org-000000?style=for-the-badge&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxZW0iIGhlaWdodD0iNzkiIHZpZXdCb3g9IjAgMCA2NyA3OSIgZmlsbD0ibm9uZSI%2BPHBhdGggZmlsbD0iI2ZmZiIgZD0iTTUyLjYxMiA3OC43ODJIMjMuMzNhMi41NTkgMi41NTkgMCAwIDEtMi41Ni0yLjU1OHYtNy42NzdoLTcuOTczYTIuNTYgMi41NiAwIDAgMS0yLjU2LTIuNTZWNTUuMzE1bC04LjgyLTQuMzk3YTIuNTU5IDIuNTU5IDAgMCAxLS45ODYtMy43MWw5LjgwNy0xNC43MTR2LTQuMzVDMTAuMjQgMTIuNTk5IDIyLjg0MyAwIDM4LjM4OCAwIDUzLjkzMiAwIDY2LjUzNCAxMi42IDY2LjUzOCAyOC4xNDNjLS42MzIgMjcuODI0LTEwLjc2IDIzLjUxNi0xMS4xOCAzNC4wNDVsLS4xODcgMTQuMDM1YTIuNTkgMi41OSAwIDAgMS0uNzUgMS44MSAyLjU1IDIuNTUgMCAwIDEtMS44MDkuNzVabS0yNi43MjMtNS4xMTdoMjQuMTY0bC4yODYtMTQuNTQyYy0uMjYzLTYuNjU2IDExLjcxNi04LjI0MyAxMS4wOC0zMC43MzQtLjM1OC0xMi43MTMtMTAuMzEzLTIzLjI3MS0yMy4wMzEtMjMuMjcxLTEyLjcxOCAwLTIzLjAyOSAxMC4zMDctMjMuMDMyIDIzLjAyNXY1LjExN2MwIC41MDYtLjE1IDEtLjQzIDEuNDJsLTguNjMgMTIuOTQxIDcuNjQ1IDMuODJhMi41NTkgMi41NTkgMCAwIDEgMS40MTUgMi4yOTF2OS42OTdoNy45NzRhMi41NTkgMi41NTkgMCAwIDEgMi41NiAyLjU1OXY3LjY3N1oiLz48cGF0aCBmaWxsPSIjZmZmIiBkPSJNNDAuMzcyIDU4LjIyMlYzOC45MzRjLjExOCAwIC4yMzcuMDE4LjM1NS4wMTggOS43NjktLjAxMiAxNy4wNS05LjAxMiAxNS4wMjItMTguNTY3YTIuMzY2IDIuMzY2IDAgMCAwLTEuODIxLTEuODIyYy04LjEwNi0xLjczLTE2LjEyMSAzLjI5Mi0xOC4wOTggMTEuMzQxLS4wMjQtLjAyNC0uMDQzLS4wNS0uMDY2LS4wNzNhMTUuMzIzIDE1LjMyMyAwIDAgMC0xNC4wNi00LjE3IDIuMzY1IDIuMzY1IDAgMCAwLTEuODIxIDEuODJjLTIuMDI4IDkuNTU1IDUuMjUyIDE4LjU1NCAxNS4wMiAxOC41NjguMjM2IDAgLjQ5Mi0uMDI4LjczOC0uMDR2MTIuMjEzaDQuNzMxWm0yLjgzOS0zMi4xNDNhMTAuNjQ2IDEwLjY0NiAwIDAgMSA4LjEyNC0zLjEwNmMuMzUgNi4zNC00Ljg4OCAxMS41NzctMTEuMjI4IDExLjIzYTEwLjU4IDEwLjU4IDAgMCAxIDMuMTA0LTguMTI0Wk0yNy40MDMgMzguMTkzYTEwLjYwNyAxMC42MDcgMCAwIDEtMy4xMTgtOC4xMjNjNi4zNDQtLjM1OCAxMS41ODcgNC44ODYgMTEuMjI4IDExLjIzLTMuMDIzLjE2OS01Ljk3My0uOTYxLTguMTEtMy4xMDdaIi8%2BPC9zdmc%2B)](https://brainmade.org/)
