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
| sharpe_ratio     | f64    | 夏普比率 |
| alpha            | f64    | 阿尔法（超额收益） |
| beta             | f64    | 贝塔（系统风险） |

## 计算公式说明

### 总收益率 (Total Return)

```
总收益率 = (期末净值 - 期初净值) / 期初净值
```

### 年化收益率 (Annualized Return)

```
年化收益率 = (1 + 总收益率)^(365 / 实际天数) - 1
```

### 波动率 (Volatility)

计算日收益率的标准差，然后年化：

```
日收益率标准差 = sqrt( Σ(Ri - R_mean)² / (n - 1) )
波动率 = 日收益率标准差 × sqrt(252)
```

其中：
- Ri = 第 i 天的日收益率
- R_mean = 日收益率平均值
- n = 交易日数量
- 252 = 一年交易日数量

### 最大回撤 (Max Drawdown)

```
回撤 = (历史最高点 - 当前净值) / 历史最高点
最大回撤 = max(所有回撤值)
```

### 夏普比率 (Sharpe Ratio)

```
夏普比率 = (年化收益率 - 无风险利率) / 波动率
```

默认无风险利率 = 3% (年化)

### 阿尔法 (Alpha)

衡量基金相对基准的超额收益能力：

```
Alpha = 基金年化收益率 - [无风险利率 + Beta × (基准年化收益率 - 无风险利率)]
```

或从日收益率计算：

```
Alpha_日 = Rp_daily - [Rf_daily + Beta × (Rm_daily - Rf_daily)]
Alpha_年化 = Alpha_日 × 252
```

其中：
- Rp_daily = 基金日收益率均值
- Rm_daily = 基准日收益率均值
- Rf_daily = 无风险日利率 = 3% / 252
- 252 = 一年交易日数量

**解读**：
- Alpha > 0：基金跑赢基准（经风险调整后）
- Alpha = 0：基金与基准表现一致
- Alpha < 0：基金跑输基准

### 贝塔 (Beta)

衡量基金相对于基准的系统风险暴露：

```
Beta = Cov(Rp, Rm) / Var(Rm)
```

其中：
- Cov(Rp, Rm) = 基金收益率与基准收益率的协方差
- Var(Rm) = 基准收益率的方差

**解读**：
- Beta = 1：基金与基准波动一致
- Beta > 1：基金比基准波动更大（激进型）
- Beta < 1：基金比基准波动更小（保守型）
- Beta < 0：基金与基准负相关（罕见）

### 协方差计算

```
Cov(X, Y) = Σ[(Xi - X_mean) × (Yi - Y_mean)] / (n - 1)
```

### 方差计算

```
Var(X) = Σ[(Xi - X_mean)²] / (n - 1)
```

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
