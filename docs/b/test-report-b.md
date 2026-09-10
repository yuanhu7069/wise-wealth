# B 期测试报告(test-report-b.md)

> 本文件由 ticket 08 起头(备份与恢复演练一节),其余章节由 ticket 09 补齐:
> AC 回链(prd-v1 §4.2 十七条)、自动化测试结果、基线 §16 审查清单、依赖审计
> (cargo audit / npm audit)、遗留清单。**未写到的章节 = 未做,不预留空表假装已完成。**

## 1. 备份与恢复演练(AC-15 / ADR-A-002 兑现)

### 1.1 备份侧:已通过(2026-09-11)

证据脚本 `.scratch/b-guide-flow/tmp-verify-08.sh`(第 1/2/3/6 节全绿,退出码 2 = 演练侧阻塞)。

| 验的什么 | 怎么验的 | 结果 |
|---|---|---|
| 转储为自定义格式且档案可读 | `pg_restore --list` 列出 CUSTOM 格式与 5 张业务表 | ✓ |
| 校验文件可用 | `sha256sum -c` 通过;**改动转储 1 字节后如实报错** | ✓ |
| 保留策略 = 日备 7 + 周备 4 | 造 20 天合成备份后跑清理,与独立写的规则实现逐文件比对 | ✓ 保留集合完全一致(7 + 4) |
| 备份不进版本库 | `git check-ignore` + `git status --porcelain` | ✓ `backups/` 已忽略,工作区看不到备份 |
| README 红线与命令 | grep 断言红线原句、备份命令、恢复演练命令 | ✓ |

备份文件(本机 `backups/`,已 gitignore):

| 文件 | 大小 |
|---|---|
| `backups/wise_wealth_20260911_0006.dump` | 20K |
| `backups/wise_wealth_20260911_0005.dump` | 20K |

### 1.2 恢复演练:未执行(2026-09-11 延后)

**结论:AC-15 未通过,本次未演练。** 不编造演练结果。

| 项 | 状态 |
|---|---|
| 演练脚本 `scripts/restore-drill.sh` | 已交付(备份 → 副本库恢复 → 10 项抽验 → 副本库起服务读方案) |
| 无权限路径 | 已验证:打印可执行 SQL 并以**退出码 3**(区别于失败码 1)结束 |
| 演练执行 | ✗ 未执行 |

**阻塞原因(实测输出)**:应用角色 `wise_wealth_user` 无建库权限(`rolcreatedb=f`)。

```
$ psql <维护库> -c 'CREATE DATABASE ww_probe_delete_me;'
ERROR:  permission denied to create database
```

(连接权限正常,只差建库权限;现存库 `testdb` 属主是 `postgres`,应用角色对它同样没有 CREATE 权限,无法顶替。)

**解除阻塞需要实例管理员执行任一条**:

```sql
CREATE DATABASE wise_wealth_backup_test OWNER wise_wealth_user;  -- OWNER 不能省(PG15 起 public schema 归库主所有)
ALTER ROLE wise_wealth_user CREATEDB;                            -- 或:让脚本自建自用,可反复演练
```

之后一条命令跑完演练并打印记录块:`scripts/restore-drill.sh`。

**影响**:①`README` 的红线「恢复演练完成前不录入不可重建的真实数据」**继续生效**;②ticket 09 的
AC-15 需按「未完成」记录;③其余 B 期数据(种子账号的验证数据)本就是可重建的,不受此限。
