import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";
import { api } from "./api";
import { errorMessage, money, parseMoney, parseQuantity, quantity, today } from "./format";
import type {
  Category,
  InventoryMovement,
  OperationSummary,
  Product,
  ProductInput,
  SaleDetail,
  UnitType,
} from "./types";

type View = "sell" | "products" | "categories" | "stock" | "purchases";
type Notice = { kind: "success" | "error"; text: string } | null;
type CartLine = { product: Product; quantityMillis: number };
type PurchaseLine = CartLine & { unitCostCents: number };

const navItems: { id: View; label: string; icon: string }[] = [
  { id: "sell", label: "Vender", icon: "▣" },
  { id: "products", label: "Productos", icon: "◇" },
  { id: "categories", label: "Categorías", icon: "⌗" },
  { id: "stock", label: "Stock", icon: "≋" },
  { id: "purchases", label: "Compras", icon: "↓" },
];

const emptyProduct: ProductInput = {
  name: "",
  barcode: null,
  categoryId: null,
  unitType: "UNIT",
  salePriceCents: 0,
};

function amountFor(quantityMillis: number, cents: number) {
  return Math.round((quantityMillis * cents) / 1000);
}

function QuantityInput({
  product,
  value,
  onChange,
}: {
  product: Product;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <input
      className="quantity-input"
      inputMode="decimal"
      aria-label={`Cantidad de ${product.name}`}
      defaultValue={product.unitType === "WEIGHT" ? (value / 1000).toFixed(3).replace(".", ",") : value / 1000}
      onChange={(event) => {
        const parsed = parseQuantity(event.target.value, product.unitType);
        if (parsed !== null) onChange(parsed);
      }}
    />
  );
}

function App() {
  const [view, setView] = useState<View>("sell");
  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [purchases, setPurchases] = useState<OperationSummary[]>([]);
  const [sales, setSales] = useState<OperationSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<Notice>(null);

  const refresh = useCallback(async () => {
    const [nextProducts, nextCategories, nextPurchases, nextSales] = await Promise.all([
      api.listProducts(),
      api.listCategories(),
      api.listPurchases(),
      api.listSales(),
    ]);
    setProducts(nextProducts);
    setCategories(nextCategories);
    setPurchases(nextPurchases);
    setSales(nextSales);
  }, []);

  useEffect(() => {
    // La carga inicial sincroniza la UI con la base local de Tauri.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    refresh()
      .catch((error) => setNotice({ kind: "error", text: errorMessage(error) }))
      .finally(() => setLoading(false));
  }, [refresh]);

  const complete = async (task: () => Promise<unknown>, message: string) => {
    try {
      await task();
      await refresh();
      setNotice({ kind: "success", text: message });
      return true;
    } catch (error) {
      setNotice({ kind: "error", text: errorMessage(error) });
      return false;
    }
  };

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">DN</span>
          <div><strong>Despensa</strong><small>Nahuel</small></div>
        </div>
        <nav aria-label="Navegación principal">
          {navItems.map((item) => (
            <button key={item.id} className={view === item.id ? "active" : ""} onClick={() => setView(item.id)}>
              <span aria-hidden="true">{item.icon}</span>{item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-note"><span className="status-dot" /> Datos guardados en esta PC</div>
      </aside>

      <main className="workspace">
        {notice && (
          <div className={`notice ${notice.kind}`} role="status">
            {notice.text}<button aria-label="Cerrar mensaje" onClick={() => setNotice(null)}>×</button>
          </div>
        )}
        {loading ? <div className="loading">Abriendo la despensa…</div> : (
          <>
            {view === "sell" && <PosView products={products} sales={sales} complete={complete} />}
            {view === "products" && <ProductsView products={products} categories={categories} complete={complete} />}
            {view === "categories" && <CategoriesView categories={categories} complete={complete} />}
            {view === "stock" && <StockView products={products} complete={complete} />}
            {view === "purchases" && <PurchasesView products={products} purchases={purchases} complete={complete} />}
          </>
        )}
      </main>
    </div>
  );
}

function PageHeader({ eyebrow, title, detail }: { eyebrow: string; title: string; detail: string }) {
  return <header className="page-header"><div><p className="eyebrow">{eyebrow}</p><h1>{title}</h1><p>{detail}</p></div></header>;
}

function CategoriesView({
  categories,
  complete,
}: {
  categories: Category[];
  complete: (task: () => Promise<unknown>, message: string) => Promise<boolean>;
}) {
  const [name, setName] = useState("");
  const [editing, setEditing] = useState<Category | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const success = await complete(
      () => editing ? api.updateCategory(editing.id, name) : api.createCategory(name),
      editing ? "Categoría actualizada." : "Categoría creada.",
    );
    if (success) { setName(""); setEditing(null); }
  };

  return <div className="page narrow-page">
    <PageHeader eyebrow="Organización" title="Categorías" detail="Una lista simple para encontrar los productos más rápido." />
    <div className="two-column">
      <form className="panel form-panel" onSubmit={submit}>
        <h2>{editing ? "Editar categoría" : "Nueva categoría"}</h2>
        <label>Nombre<input autoFocus value={name} onChange={(event) => setName(event.target.value)} placeholder="Ej. Bebidas" /></label>
        <div className="actions"><button className="primary" disabled={!name.trim()}>{editing ? "Guardar cambios" : "Crear categoría"}</button>
          {editing && <button type="button" className="ghost" onClick={() => { setEditing(null); setName(""); }}>Cancelar</button>}
        </div>
      </form>
      <section className="panel"><div className="panel-heading"><h2>Categorías</h2><span className="badge">{categories.length}</span></div>
        {categories.length === 0 ? <Empty text="Todavía no hay categorías." /> : <div className="simple-list">{categories.map((category) =>
          <button key={category.id} onClick={() => { setEditing(category); setName(category.name); }}><span>{category.name}</span><small>Editar →</small></button>)}</div>}
      </section>
    </div>
  </div>;
}

function ProductsView({
  products,
  categories,
  complete,
}: {
  products: Product[];
  categories: Category[];
  complete: (task: () => Promise<unknown>, message: string) => Promise<boolean>;
}) {
  const [search, setSearch] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState<ProductInput>(emptyProduct);
  const [price, setPrice] = useState("");
  const [showForm, setShowForm] = useState(false);
  const filtered = products.filter((product) => `${product.name} ${product.barcode ?? ""}`.toLowerCase().includes(search.toLowerCase()));

  const startCreate = () => { setEditingId(null); setForm(emptyProduct); setPrice(""); setShowForm(true); };
  const startEdit = (product: Product) => {
    setEditingId(product.id);
    setForm({ name: product.name, barcode: product.barcode, categoryId: product.categoryId, unitType: product.unitType, salePriceCents: product.salePriceCents });
    setPrice((product.salePriceCents / 100).toFixed(2).replace(".", ","));
    setShowForm(true);
  };
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const cents = parseMoney(price);
    if (cents === null) return;
    const input = { ...form, barcode: form.barcode || null, salePriceCents: cents };
    const success = await complete(
      () => editingId ? api.updateProduct(editingId, input) : api.createProduct(input),
      editingId ? "Producto actualizado." : "Producto creado.",
    );
    if (success) setShowForm(false);
  };

  return <div className="page">
    <PageHeader eyebrow="Catálogo" title="Productos" detail="Precios, costos y stock actual en una sola vista." />
    <div className="toolbar"><input className="search" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Buscar por nombre o código…" /><button className="primary" onClick={startCreate}>+ Nuevo producto</button></div>
    {showForm && <form className="panel product-form" onSubmit={submit}>
      <div className="panel-heading"><h2>{editingId ? "Editar producto" : "Nuevo producto"}</h2><button type="button" className="close" onClick={() => setShowForm(false)}>×</button></div>
      <div className="form-grid">
        <label>Nombre<input autoFocus value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></label>
        <label>Código de barras<input value={form.barcode ?? ""} onChange={(e) => setForm({ ...form, barcode: e.target.value })} /></label>
        <label>Categoría<select value={form.categoryId ?? ""} onChange={(e) => setForm({ ...form, categoryId: e.target.value ? Number(e.target.value) : null })}><option value="">Sin categoría</option>{categories.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}</select></label>
        <label>Tipo<select value={form.unitType} onChange={(e) => setForm({ ...form, unitType: e.target.value as UnitType })}><option value="UNIT">Por unidad</option><option value="WEIGHT">Por peso (kg)</option></select></label>
        <label>Precio de venta<input inputMode="decimal" value={price} onChange={(e) => setPrice(e.target.value)} placeholder="1600,00" /></label>
      </div>
      <div className="actions"><button className="primary" disabled={!form.name.trim() || parseMoney(price) === null}>Guardar producto</button><button type="button" className="ghost" onClick={() => setShowForm(false)}>Cancelar</button></div>
    </form>}
    <section className="panel table-panel">
      {filtered.length === 0 ? <Empty text="No hay productos para mostrar." /> : <div className="table-wrap"><table><thead><tr><th>Producto</th><th>Categoría</th><th>Tipo</th><th>Stock</th><th>Costo</th><th>Precio</th><th>Margen</th><th /></tr></thead><tbody>
        {filtered.map((product) => {
          const gain = product.salePriceCents - product.currentCostCents;
          const margin = product.salePriceCents > 0 ? (gain / product.salePriceCents) * 100 : 0;
          return <tr key={product.id} className={!product.active ? "muted-row" : ""}>
            <td><strong>{product.name}</strong><small>{product.barcode || "Sin código"}{!product.active && " · Inactivo"}</small></td><td>{product.categoryName || "—"}</td><td>{product.unitType === "UNIT" ? "Unidad" : "Peso"}</td><td>{quantity(product.stockMillis, product.unitType)}</td><td>{money(product.currentCostCents)}</td><td>{money(product.salePriceCents)}</td><td>{money(gain)}<small>{margin.toFixed(1).replace(".", ",")}%</small></td>
            <td><div className="row-actions"><button onClick={() => startEdit(product)}>Editar</button><button onClick={() => complete(() => api.setProductActive(product.id, !product.active), product.active ? "Producto desactivado." : "Producto activado.")}>{product.active ? "Desactivar" : "Activar"}</button></div></td>
          </tr>;
        })}
      </tbody></table></div>}
    </section>
  </div>;
}

function StockView({
  products,
  complete,
}: {
  products: Product[];
  complete: (task: () => Promise<unknown>, message: string) => Promise<boolean>;
}) {
  const [selectedId, setSelectedId] = useState<number | null>(products[0]?.id ?? null);
  const [mode, setMode] = useState<"initial" | "adjust">("initial");
  const [value, setValue] = useState("");
  const [note, setNote] = useState("");
  const [movements, setMovements] = useState<InventoryMovement[]>([]);
  const selected = products.find((product) => product.id === selectedId) ?? null;

  useEffect(() => {
    api.listMovements(selectedId).then(setMovements).catch(() => setMovements([]));
  }, [selectedId, products]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!selected) return;
    const millis = parseQuantity(value, selected.unitType);
    if (millis === null && !(mode === "adjust" && value.trim() === "0")) return;
    const quantityMillis = mode === "adjust" && value.trim() === "0" ? 0 : millis!;
    const success = await complete(
      () => mode === "initial"
        ? api.addInitialStock(selected.id, quantityMillis, today(), note || null)
        : api.adjustStock(selected.id, quantityMillis, today(), note || null),
      mode === "initial" ? "Stock inicial registrado." : "Ajuste registrado.",
    );
    if (success) { setValue(""); setNote(""); setMovements(await api.listMovements(selected.id)); }
  };

  return <div className="page">
    <PageHeader eyebrow="Inventario" title="Stock" detail="Cada cambio queda registrado como un movimiento auditable." />
    <div className="stock-summary">{products.map((product) => <button key={product.id} className={selectedId === product.id ? "selected" : ""} onClick={() => setSelectedId(product.id)}><span>{product.name}</span><strong>{quantity(product.stockMillis, product.unitType)} {product.unitType === "WEIGHT" ? "kg" : "u."}</strong><small>{money(product.currentCostCents)} c/u · valor {money(amountFor(product.stockMillis, product.currentCostCents))}</small></button>)}</div>
    {selected && <div className="two-column stock-detail">
      <form className="panel form-panel" onSubmit={submit}><h2>Actualizar {selected.name}</h2>
        <div className="segmented"><button type="button" className={mode === "initial" ? "active" : ""} onClick={() => setMode("initial")}>Stock inicial</button><button type="button" className={mode === "adjust" ? "active" : ""} onClick={() => setMode("adjust")}>Ajuste</button></div>
        <p className="helper">{mode === "initial" ? "Disponible sólo antes del primer movimiento." : "Ingresá la cantidad real contada; guardaremos la diferencia."}</p>
        <label>{mode === "initial" ? "Cantidad inicial" : "Cantidad real"}<input inputMode="decimal" value={value} onChange={(e) => setValue(e.target.value)} placeholder={selected.unitType === "WEIGHT" ? "0,750" : "10"} /></label>
        <label>Nota opcional<textarea value={note} onChange={(e) => setNote(e.target.value)} placeholder="Motivo o referencia" /></label>
        <button className="primary">Registrar movimiento</button>
      </form>
      <section className="panel"><div className="panel-heading"><h2>Movimientos recientes</h2><span className="badge">{movements.length}</span></div>
        {movements.length === 0 ? <Empty text="Este producto todavía no tiene movimientos." /> : <div className="movement-list">{movements.map((movement) => <article key={movement.id}><span className={`movement-icon ${movement.quantityMillis > 0 ? "positive" : "negative"}`}>{movement.quantityMillis > 0 ? "+" : "−"}</span><div><strong>{movement.movementType.replace("_", " ")}</strong><small>{movement.occurredAt}{movement.note ? ` · ${movement.note}` : ""}</small></div><b>{movement.quantityMillis > 0 ? "+" : ""}{quantity(movement.quantityMillis, selected.unitType)}</b></article>)}</div>}
      </section>
    </div>}
  </div>;
}

function PurchasesView({
  products,
  purchases,
  complete,
}: {
  products: Product[];
  purchases: OperationSummary[];
  complete: (task: () => Promise<unknown>, message: string) => Promise<boolean>;
}) {
  const active = products.filter((product) => product.active);
  const [selectedId, setSelectedId] = useState<number | null>(active[0]?.id ?? null);
  const [lines, setLines] = useState<PurchaseLine[]>([]);
  const total = lines.reduce((sum, line) => sum + amountFor(line.quantityMillis, line.unitCostCents), 0);

  const add = () => {
    const product = products.find((item) => item.id === selectedId);
    if (!product || lines.some((line) => line.product.id === product.id)) return;
    setLines([...lines, { product, quantityMillis: 1000, unitCostCents: product.currentCostCents }]);
  };
  const confirm = async () => {
    const success = await complete(
      () => api.confirmPurchase(today(), lines.map((line) => ({ productId: line.product.id, quantityMillis: line.quantityMillis, unitCostCents: line.unitCostCents }))),
      "Compra confirmada: stock y costos actualizados.",
    );
    if (success) setLines([]);
  };

  return <div className="page">
    <PageHeader eyebrow="Mercadería" title="Compras" detail="Confirmar una compra aumenta stock y recalcula el costo promedio." />
    <div className="purchase-layout">
      <section className="panel purchase-editor"><div className="panel-heading"><h2>Nueva compra</h2><span className="badge">{today()}</span></div>
        <div className="add-line"><select value={selectedId ?? ""} onChange={(e) => setSelectedId(Number(e.target.value))}><option value="">Elegir producto</option>{active.map((product) => <option key={product.id} value={product.id}>{product.name}</option>)}</select><button className="secondary" onClick={add}>Agregar</button></div>
        {lines.length === 0 ? <Empty text="Agregá productos para registrar una compra." /> : <div className="line-list">{lines.map((line) => <article key={line.product.id}><div><strong>{line.product.name}</strong><small>Stock actual: {quantity(line.product.stockMillis, line.product.unitType)}</small></div><label>Cantidad<QuantityInput product={line.product} value={line.quantityMillis} onChange={(value) => setLines(lines.map((item) => item.product.id === line.product.id ? { ...item, quantityMillis: value } : item))} /></label><label>Costo unitario<input inputMode="decimal" defaultValue={(line.unitCostCents / 100).toFixed(2).replace(".", ",")} onChange={(e) => { const cents = parseMoney(e.target.value); if (cents !== null) setLines(lines.map((item) => item.product.id === line.product.id ? { ...item, unitCostCents: cents } : item)); }} /></label><b>{money(amountFor(line.quantityMillis, line.unitCostCents))}</b><button className="remove" onClick={() => setLines(lines.filter((item) => item.product.id !== line.product.id))}>×</button></article>)}</div>}
        <footer className="total-bar"><div><span>Total</span><strong>{money(total)}</strong></div><button className="primary large" disabled={lines.length === 0} onClick={confirm}>Confirmar compra</button></footer>
      </section>
      <section className="panel history"><div className="panel-heading"><h2>Últimas compras</h2></div>{purchases.length === 0 ? <Empty text="No hay compras confirmadas." /> : purchases.map((purchase) => <article key={purchase.id}><div><strong>Compra #{purchase.id}</strong><small>{purchase.occurredAt} · {purchase.itemCount} productos</small></div><b>{money(purchase.totalCents)}</b></article>)}</section>
    </div>
  </div>;
}

function PosView({
  products,
  sales,
  complete,
}: {
  products: Product[];
  sales: OperationSummary[];
  complete: (task: () => Promise<unknown>, message: string) => Promise<boolean>;
}) {
  const [query, setQuery] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const [saleDetail, setSaleDetail] = useState<SaleDetail | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const available = useMemo(() => products.filter((product) => product.active && `${product.name} ${product.barcode ?? ""}`.toLowerCase().includes(query.toLowerCase())).slice(0, 12), [products, query]);
  const total = cart.reduce((sum, line) => sum + amountFor(line.quantityMillis, line.product.salePriceCents), 0);

  const add = (product: Product) => {
    if (!product.active) {
      void complete(() => Promise.reject(`${product.name} está inactivo.`), "");
      return;
    }
    if (product.stockMillis <= 0) {
      void complete(() => Promise.reject(`${product.name} no tiene stock disponible.`), "");
      return;
    }
    const existing = cart.find((line) => line.product.id === product.id);
    const increment = 1000;
    if (existing) {
      if (existing.quantityMillis + increment > product.stockMillis) {
        void complete(() => Promise.reject(`Stock insuficiente para ${product.name}.`), "");
        return;
      }
      setCart(cart.map((line) => line.product.id === product.id ? { ...line, quantityMillis: line.quantityMillis + increment } : line));
    } else {
      setCart([...cart, { product, quantityMillis: Math.min(increment, product.stockMillis) }]);
    }
    setQuery("");
    searchRef.current?.focus();
  };
  const scan = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key !== "Enter") return;
    const exact = products.find((product) => product.barcode?.toLowerCase() === query.trim().toLowerCase());
    if (exact) add(exact);
    else void complete(() => Promise.reject("No existe un producto con ese código de barras."), "");
  };
  const confirm = async () => {
    const success = await complete(
      () => api.confirmSale(today(), cart.map((line) => ({ productId: line.product.id, quantityMillis: line.quantityMillis, unitCostCents: null }))),
      "Venta confirmada y stock descontado.",
    );
    if (success) setCart([]);
  };
  const openSale = async (id: number) => {
    try { setSaleDetail(await api.getSale(id)); } catch { setSaleDetail(null); }
  };

  return <div className="page pos-page">
    <PageHeader eyebrow="Punto de venta" title="Nueva venta" detail="Buscá por nombre o escaneá un código y presioná Enter." />
    <div className="pos-layout">
      <section className="catalog-panel">
        <input ref={searchRef} autoFocus className="pos-search" value={query} onChange={(e) => setQuery(e.target.value)} onKeyDown={scan} placeholder="⌕  Buscar producto o escanear código…" />
        <div className="product-grid">{available.map((product) => <button key={product.id} disabled={product.stockMillis <= 0} onClick={() => add(product)}><span className="product-category">{product.categoryName || "Producto"}</span><strong>{product.name}</strong><small>Stock: {quantity(product.stockMillis, product.unitType)} {product.unitType === "WEIGHT" ? "kg" : "u."}</small><b>{money(product.salePriceCents)}</b>{product.stockMillis <= 0 && <em>Sin stock</em>}</button>)}</div>
        {available.length === 0 && <Empty text="No encontramos productos activos con esa búsqueda." />}
        {sales.length > 0 && <div className="recent-sales"><h2>Ventas recientes</h2>{sales.slice(0, 5).map((sale) => <button key={sale.id} onClick={() => openSale(sale.id)}><span>Venta #{sale.id} · {sale.occurredAt}</span><strong>{money(sale.totalCents)}</strong></button>)}</div>}
      </section>
      <aside className="cart-panel"><div className="panel-heading"><h2>Venta actual</h2><span className="badge">{cart.length} items</span></div>
        {cart.length === 0 ? <Empty text="El carrito está vacío." /> : <div className="cart-lines">{cart.map((line) => <article key={line.product.id}><div><strong>{line.product.name}</strong><small>{money(line.product.salePriceCents)} × {quantity(line.quantityMillis, line.product.unitType)}</small></div><QuantityInput product={line.product} value={line.quantityMillis} onChange={(value) => { if (value <= line.product.stockMillis) setCart(cart.map((item) => item.product.id === line.product.id ? { ...item, quantityMillis: value } : item)); }} /><b>{money(amountFor(line.quantityMillis, line.product.salePriceCents))}</b><button className="remove" onClick={() => setCart(cart.filter((item) => item.product.id !== line.product.id))}>×</button></article>)}</div>}
        <footer className="checkout"><div><span>Total</span><strong>{money(total)}</strong></div><button className="primary checkout-button" disabled={cart.length === 0} onClick={confirm}>Confirmar venta</button><small>Se validará el stock antes de guardar.</small></footer>
      </aside>
    </div>
    {saleDetail && <div className="modal-backdrop" onClick={() => setSaleDetail(null)}><section className="modal" onClick={(e) => e.stopPropagation()}><div className="panel-heading"><div><p className="eyebrow">Historial</p><h2>Venta #{saleDetail.id}</h2></div><button className="close" onClick={() => setSaleDetail(null)}>×</button></div><p>{saleDetail.occurredAt}</p>{saleDetail.items.map((item) => <article className="snapshot" key={item.productId}><div><strong>{item.productName}</strong><small>{quantity(item.quantityMillis, products.find((p) => p.id === item.productId)?.unitType ?? "UNIT")} × {money(item.unitPriceCents)}</small></div><div><b>{money(item.subtotalCents)}</b><small>Costo histórico: {money(item.unitCostCents)}</small></div></article>)}<div className="snapshot-total"><span>Total {money(saleDetail.totalCents)}</span><span>Costo {money(saleDetail.totalCostCents)}</span><strong>Margen bruto {money(saleDetail.totalCents - saleDetail.totalCostCents)}</strong></div></section></div>}
  </div>;
}

function Empty({ text }: { text: string }) {
  return <div className="empty"><span>◎</span><p>{text}</p></div>;
}

export default App;
