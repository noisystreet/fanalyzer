# Data Model

## Core Entities

### Fund

基金基本信息。

| Field      | Type     | Description |
|------------|----------|-------------|
| code       | String   | 基金代码（唯一标识） |
| name       | String   | 基金名称 |
| fund_type  | FundType | 基金类型 |

### FundType (Enum)

| Variant  | Description |
|----------|-------------|
| Stock    | 股票型 |
| Bond     | 债券型 |
| Hybrid   | 混合型 |
| Index    | 指数型 |
| Monetary | 货币型 |
| QDII     | QDII |
| Other    | 其他 |

### FundNav

基金净值数据点。

| Field        | Type          | Description |
|--------------|---------------|-------------|
| code         | String        | 基金代码 |
| date         | NaiveDate     | 净值日期 |
| nav          | f64           | 单位净值 |
| acc_nav      | f64           | 累计净值 |
| daily_return | Option\<f64\> | 日收益率 |

### FundAnalysis

基金分析结果。

| Field            | Type  | Description |
|------------------|-------|-------------|
| code             | String | 基金代码 |
| period_days      | u32    | 分析周期（天） |
| avg_nav          | f64    | 平均净值 |
| max_nav          | f64    | 最大净值 |
| min_nav          | f64    | 最小净值 |
| total_return     | f64    | 总收益率 |
| annualized_return| f64    | 年化收益率 |
| volatility       | f64    | 波动率 |
| max_drawdown     | f64    | 最大回撤 |

## Entity Relationships

```
Fund 1──* FundNav (一个基金有多条净值记录)
Fund 1──* FundAnalysis (一个基金可有多个周期分析结果)
```

## Data Flow

```
API/DataSource → FundNav[] → FundAnalysis → CLI Output / Report
                    ↓
                 Fund (metadata)
```
