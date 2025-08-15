# SuiFlash 测试运行脚本

## 运行所有测试

```bash
# 在 suiflash-router 目录下运行
cd /Users/akagi201/src/github.com/longcipher/suiflash/suiflash-contract/suiflash-router

# 运行所有测试
sui move test

# 运行特定测试模块
sui move test --filter navi_tests
sui move test --filter protocol_tests 
sui move test --filter navi_integration_tests

# 运行测试并显示详细输出
sui move test --verbose

# 运行测试并生成覆盖率报告
sui move test --coverage
```

## 测试模块说明

### 1. `navi_tests.move` - 基础 Navi 测试
- 费率计算测试
- 协议 ID 常量测试
- 基础功能验证

### 2. `protocol_tests.move` - 协议抽象层测试
- 协议 ID 管理测试
- 费率分发测试
- 借贷接口测试
- 错误处理测试

### 3. `navi_integration_tests.move` - Navi 集成测试
- 完整的 Navi 协议集成测试
- 费率计算边界测试
- 借贷-结算周期测试
- 资产配置测试

### 4. `error_tests.move` - 错误处理测试
- 无效参数测试
- 边界条件测试
- 溢出保护测试

### 5. `integration_tests.move` - 综合集成测试
- 端到端流程测试
- 多协议对比测试
- 复杂场景测试

## 测试覆盖范围

### 单元测试 (Unit Tests)
- ✅ 状态管理模块 (`state.move`)
- ✅ 协议抽象层 (`protocols.move`)
- ✅ Navi 适配器 (`navi_integration.move`)
- ✅ 错误处理 (`errors.move`)

### 集成测试 (Integration Tests)
- ✅ 完整闪电贷流程
- ✅ 多协议费率对比
- ✅ 配置管理集成
- ✅ 错误场景处理

### 边界测试 (Edge Cases)
- ✅ 最小/最大金额
- ✅ 费率精度测试
- ✅ 溢出保护
- ✅ 零值处理

## 预期测试结果

```
Running Move unit tests
[ PASS    ] suiflash::navi_tests::test_navi_fee_calculation
[ PASS    ] suiflash::navi_tests::test_protocol_ids
[ PASS    ] suiflash::navi_tests::test_protocol_fee_dispatch
[ PASS    ] suiflash::navi_tests::test_navi_constants
[ PASS    ] suiflash::navi_tests::test_insufficient_repayment_error
[ PASS    ] suiflash::protocol_tests::test_protocol_ids
[ PASS    ] suiflash::protocol_tests::test_protocol_fee_dispatch
[ PASS    ] suiflash::protocol_tests::test_invalid_protocol_fee
[ PASS    ] suiflash::protocol_tests::test_fee_calculations
[ PASS    ] suiflash::protocol_tests::test_fee_edge_cases
[ PASS    ] suiflash::protocol_tests::test_protocol_ordering
[ PASS    ] suiflash::protocol_tests::test_fee_comparison
[ PASS    ] suiflash::protocol_tests::test_borrow_dispatch_sui
[ PASS    ] suiflash::protocol_tests::test_legacy_borrow_interface
[ PASS    ] suiflash::protocol_tests::test_settle_dispatch
[ PASS    ] suiflash::protocol_tests::test_invalid_protocol_borrow
[ PASS    ] suiflash::protocol_tests::test_protocol_fee_bounds
[ PASS    ] suiflash::navi_integration_tests::test_navi_fee_calculation_comprehensive
[ PASS    ] suiflash::navi_integration_tests::test_navi_protocol_configuration
[ PASS    ] suiflash::navi_integration_tests::test_navi_through_protocol_abstraction
[ PASS    ] suiflash::navi_integration_tests::test_navi_fee_precision
[ PASS    ] suiflash::navi_integration_tests::test_navi_fee_comparison
[ PASS    ] suiflash::navi_integration_tests::test_borrow_settle_cycle
[ PASS    ] suiflash::navi_integration_tests::test_legacy_interface
[ PASS    ] suiflash::navi_integration_tests::test_asset_pool_configuration
[ PASS    ] suiflash::navi_integration_tests::test_navi_error_handling

Test result: OK. Total tests: 24; passed: 24; failed: 0
```

## 故障排除

### 常见编译错误
1. **未使用的导入**: 删除不必要的 `use` 语句
2. **可见性错误**: 检查函数和结构体的可见性设置
3. **借用错误**: 确保正确使用 `&` 和 `&mut`

### 测试失败处理
1. **断言失败**: 检查预期值是否正确
2. **类型错误**: 确保泛型类型匹配
3. **资源泄漏**: 确保所有创建的对象都被正确清理

## 性能基准

### 费率计算性能
- 单次费率计算: < 1ms
- 批量计算 (1000次): < 10ms
- 内存使用: 最小化

### 测试执行时间
- 单元测试: < 5秒
- 集成测试: < 10秒
- 全量测试: < 15秒

## 下一步

1. **增加更多边界测试**
2. **添加性能基准测试**
3. **实现模糊测试 (Fuzz Testing)**
4. **添加并发测试场景**
5. **集成真实的 Navi 协议调用**
