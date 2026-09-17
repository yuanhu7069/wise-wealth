/**
 * 首页 hero 渐变 mesh(DESIGN.md「Gradient Mesh Backdrop」)。
 *
 * 参考件的招牌元素,但按本项目调整清单只用在 P01 首页(spec §2.2#11)。
 * 实现约束:参考件明确「用 SVG/大图实现,不用 CSS 渐变模拟」——这里是内联 SVG 的
 * 模糊椭圆叠出有机色带;色站只用已 token 化的五个参考件色值
 * (canvas-cream / lemon / magenta / primary / ruby),柔和感靠大半径模糊 + 低透明度。
 *
 * 几何约束(2026-09-17 走查教训):相邻色斑圆心距必须远小于半径,让相邻斑大幅重叠,
 * 否则斑间露出底色会形成硬接缝;深色斑(lemon)只允许低透明度点缀,高透明度会浑浊发脏。
 * 整块 aria-hidden:纯装饰,对读屏器与键盘不可见。
 * 颜色经 var(--color-*) 引用,暗色模式随 token 机械翻转(暗色下是深靛调氛围带)。
 */
export function MeshBackdrop() {
  return (
    <div aria-hidden="true" className="absolute inset-0 overflow-hidden bg-canvas-soft">
      <svg
        className="h-full w-full"
        viewBox="0 0 1200 320"
        preserveAspectRatio="xMidYMid slice"
        role="presentation"
      >
        <defs>
          <filter id="mesh-blur" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur stdDeviation="90" />
          </filter>
        </defs>
        <g filter="url(#mesh-blur)">
          <ellipse
            cx="80"
            cy="170"
            rx="430"
            ry="260"
            style={{ fill: "var(--color-canvas-cream)" }}
          />
          <ellipse
            cx="360"
            cy="80"
            rx="320"
            ry="170"
            style={{ fill: "var(--color-lemon)", opacity: 0.3 }}
          />
          <ellipse
            cx="620"
            cy="200"
            rx="330"
            ry="180"
            style={{ fill: "var(--color-magenta)", opacity: 0.26 }}
          />
          <ellipse
            cx="880"
            cy="100"
            rx="370"
            ry="200"
            style={{ fill: "var(--color-primary)", opacity: 0.4 }}
          />
          <ellipse
            cx="1140"
            cy="220"
            rx="330"
            ry="180"
            style={{ fill: "var(--color-ruby)", opacity: 0.3 }}
          />
        </g>
      </svg>
    </div>
  );
}
