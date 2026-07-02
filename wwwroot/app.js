// ── State ────────────────────────────────────────────────────────────
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s||"").replaceAll("&","&amp;").replaceAll("<","&lt;").replaceAll(">","&gt;");
const S = { page:"service", config:null, managed:{rule_groups:[],client_rules:[],ip_rules:[],routing_items:[],domain_sets:[],ip_sets:[],_deleted:[]}, upstreams:[], confFiles:[], includeFiles:[], extras:{} };

const META = {
  service:    {t:"基础设置",  d:"监听、测速、代理、hosts、证书" },
  upstreams:  {t:"上游 DNS", d:"6 种协议 + 15 个选项" },
  performance:{t:"缓存与性能",d:"cache-size、TTL、双栈" },
  routing:    {t:"上游分流", d:"nameserver — 域名走哪个上游组" },
  rules:      {t:"规则",     d:"domain-rules 域名行为 + IP 规则" },
  sets:       {t:"集合管理", d:"domain-set + ip-set 名称和文件路径" },
  logging:    {t:"日志与审计",d:"log-* / audit-*" },
  advanced:   {t:"高级网络",  d:"ECS、DNS64、DDNS、force-*" },
  preview:    {t:"配置预览",  d:"smartdns.conf 主配置与 plus-ui include 合成后的预览，点击保存并重启后生效" },
};

// ── API ───────────────────────────────────────────────────────────────
function ah() { const t=localStorage.getItem("sp_token"); return t?{Authorization:t}:{}; }
async function api(url,o={}) {
  const r=await fetch(url,{...o,headers:{"Content-Type":"application/json",...ah(),...(o.headers||{})}});
  if(!r.ok){const t=await r.text();throw new Error(t||`HTTP ${r.status}`);}
  const t=await r.text(); return t?JSON.parse(t):{};
}

// ── Auth ──────────────────────────────────────────────────────────────
async function login() {
  const r=await api("/api/auth/login",{method:"POST",body:JSON.stringify({username:$("username").value.trim(),password:$("password").value})});
  localStorage.setItem("sp_token",`Bearer ${r.token}`); showApp();
}
async function logout(){localStorage.removeItem("sp_token");showLogin();$("nav").classList.add("hidden");$("sidebarFoot").classList.add("hidden");$("main").classList.add("hidden");}
async function checkAuth(){try{await api("/api/auth/check");showApp();}catch(_){showLogin();}}

function showLogin(){ $("loginOverlay").classList.remove("hidden"); }

function showApp(){
  $("loginOverlay").classList.add("hidden");$("nav").classList.remove("hidden");$("sidebarFoot").classList.remove("hidden");$("main").classList.remove("hidden");
  loadAll();
}

function parseConfFilesFromText(text){
  const files=[];
  if(!text)return files;
  for(const raw of text.split("\n")){
    const line=raw.trim();
    if(!line||line.startsWith("#")||!line.startsWith("conf-file "))continue;
    const body=line.substring("conf-file ".length).trim();
    if(!body)continue;
    const parts=body.split(/\s+/);
    const path=parts[0];
    if(!path)continue;
    let group="";
    for(let i=1;i<parts.length;i++){
      if(parts[i]==="-group"&&parts[i+1]){group=parts[i+1];break;}
    }
    const managed=path.includes("/plus-ui-basic.conf")||path.includes("/plus-ui-upstreams.conf")||path.includes("/plus-ui-rules.conf");
    files.push({path,group,managed});
  }
  return files;
}

function collectConfFiles(...texts){
  const seen=new Set(), items=[];
  texts.forEach(text=>{
    parseConfFilesFromText(text).forEach(cf=>{
      const key=`${cf.path}|${cf.group}|${cf.managed?"managed":"custom"}`;
      if(seen.has(key))return;
      seen.add(key);
      items.push(cf);
    });
  });
  return items;
}

function tokenizeArgs(text){
  return (text||"").match(/(?:[^\s"]+|"[^"]*")+/g)||[];
}

function stripQuotes(text){
  if(!text)return "";
  return text.startsWith('"')&&text.endsWith('"')?text.slice(1,-1):text;
}

function parseManagedUpstream(kind,value,options){
  const item={
    kind:kind==="server-h3"?"server-http3":kind,
    value:value||"",
    groups:[],
    exclude_default_group:false,
    proxy_name:"",
    bootstrap_dns:"",
    host_ip:"",
    subnet:"",
    fallback:false,
    options:"",
    note:""
  };
  const passthrough=[];
  const tokens=tokenizeArgs(options);
  for(let i=0;i<tokens.length;i++){
    const t=tokens[i];
    if((t==="-g"||t==="-group")&&tokens[i+1]){item.groups.push(stripQuotes(tokens[++i]));continue;}
    if(t==="-e"||t==="-exclude-default-group"){item.exclude_default_group=true;continue;}
    if(t==="-proxy"&&tokens[i+1]){item.proxy_name=stripQuotes(tokens[++i]);continue;}
    if((t==="-b"||t==="-bootstrap-dns")&&tokens[i+1]){item.bootstrap_dns=stripQuotes(tokens[++i]);continue;}
    if(t==="-host-ip"&&tokens[i+1]){item.host_ip=stripQuotes(tokens[++i]);continue;}
    if(t==="-subnet"&&tokens[i+1]){item.subnet=stripQuotes(tokens[++i]);continue;}
    if(t==="-fallback"){item.fallback=true;continue;}
    passthrough.push(t);
    if(tokens[i+1]&&!tokens[i+1].startsWith("-"))passthrough.push(tokens[++i]);
  }
  item.options=passthrough.map(stripQuotes).join(" ");
  return item;
}

function parseManagedUpstreamsText(text){
  const items=[];
  if(!text)return items;
  for(const raw of text.split("\n")){
    const line=raw.trim();
    if(!line||line.startsWith("#"))continue;
    const m=line.match(/^(server|server-tcp|server-tls|server-https|server-quic|server-http3|server-h3)\s+(\S+)(?:\s+(.*))?$/);
    if(!m)continue;
    items.push(parseManagedUpstream(m[1],stripQuotes(m[2]),m[3]||""));
  }
  return items;
}

// ── Nav ───────────────────────────────────────────────────────────────
function goto(p){
  S.page=p; const m=META[p]||{};
  $("pageTitle").textContent=m.t;$("pageDesc").textContent=m.d||"";
  document.querySelectorAll(".nav-item").forEach(e=>e.classList.toggle("active",e.dataset.page===p));
  document.querySelectorAll(".page").forEach(e=>e.classList.toggle("hidden",e.id!==`page-${p}`));
  if(p==="preview")renderPreview().catch(e=>$("previewText").value="加载失败: "+e.message);
  if(p==="routing"){populateRoutingDropdowns();onRtFormChange();}
  if(p==="rules"){populateRoutingDropdowns();renderAddrCname();renderDomainRules();renderIpRules();}
}
document.querySelectorAll(".nav-item").forEach(e=>{e.onclick=()=>goto(e.dataset.page);});

// ── Load ──────────────────────────────────────────────────────────────
async function loadAll(){
  const d=await api("/api/plus/config");
  S.config=d; S.managed=d.managed_assets||{rule_groups:[],client_rules:[],ip_rules:[],routing_items:[],domain_sets:[],ip_sets:[],_deleted:[]};
  S.upstreams=parseManagedUpstreamsText(d.managed_upstreams_text);
  S.extras=(d.beginner&&d.beginner.extras)?d.beginner.extras:{};
  S.includeFiles=collectConfFiles(d.main_config_text,d.managed_basic_text);
  S.confFiles=S.includeFiles.filter(cf=>!cf.managed);
  $("statusLabel").textContent=`已连接 — ${d.config_file}`;
  fillBasic(d.beginner); fillExtras(); renderAll(); refreshGroupSelects(); populateRoutingDropdowns();
}

function fillBasic(b){if(!b)return;
  $("s_server_name").value=b.server_name||"";
  // Extract port from bind address, e.g. "[::]:11153" → "11153"
  const bindPort = (b.binds||[]).map(x=>{const m=x.match(/:(\d+)$/);return m?m[1]:"";}).filter(Boolean).join("\n");
  $("s_bind").value=bindPort||"11153";
  $("s_mdns_lookup").checked=!!b.mdns_lookup;
  $("p_cache_size").value=b.cache_size||"";
  $("p_serve_expired_ttl").value=b.serve_expired_ttl||"";
  $("p_response_mode").value=b.response_mode||"first-ping";
  $("p_speed_check_mode").value=b.speed_check_mode||"";
  $("p_prefetch_domain").checked=!!b.prefetch_domain;
  $("p_serve_expired").checked=!!b.serve_expired;
  $("p_dualstack").checked=!!b.dualstack_ip_selection;
  $("l_log_level").value=b.log_level||"info";
  $("l_audit_enable").checked=!!b.audit_enable;
}

function fillExtras(){
  const e=S.extras;
  // Extract port from bind-tcp
  const tcpMatch = (e["bind-tcp"]||"").match(/:(\d+)$/);
  $("s_bind_tcp").value=tcpMatch?tcpMatch[1]:"";
  $("s_user").value=e["user"]||"";
  $("s_socket_buff").value=e["socket-buff-size"]||"";
  $("s_no_daemon").checked=e["no-daemon"]==="yes"||e["no-daemon"]==="";
  $("s_restart_on_crash").checked=e["restart-on-crash"]==="yes"||e["restart-on-crash"]==="";
  $("s_expand_ptr").checked=e["expand-ptr-from-address"]==="yes"||e["expand-ptr-from-address"]==="";
  $("p_rr_ttl_min").value=e["rr-ttl-min"]||"";
  $("p_rr_ttl_max").value=e["rr-ttl-max"]||"";
  $("p_rr_ttl_reply_max").value=e["rr-ttl-reply-max"]||"";
  $("p_local_ttl").value=e["local-ttl"]||"";
  $("p_max_reply_ip").value=e["max-reply-ip-num"]||"";
  $("p_max_query_limit").value=e["max-query-limit"]||"";
  $("p_tcp_idle_time").value=e["tcp-idle-time"]||"";
  $("p_serve_expired_reply_ttl").value=e["serve-expired-reply-ttl"]||"";
  $("p_serve_expired_prefetch").value=e["serve-expired-prefetch-time"]||"";
  $("p_dualstack_threshold").value=e["dualstack-ip-selection-threshold"]||"";
  $("p_cache_persist").value=e["cache-persist"]||"";
  $("p_cache_file").value=e["cache-file"]||"";
  // Parse proxy: "socks5://1.2.3.4:1080 -name proxy" → name="proxy", addr="socks5://1.2.3.4:1080"
  const pm=(e["proxy-server"]||"").match(/^(.+?)\s+-name\s+(.+)$/);
  $("x_proxy_addr").value=pm?pm[1]:(e["proxy-server"]||"");
  $("x_proxy_name").value=pm?pm[2]:"";
  $("x_hosts_file").value=e["hosts-file"]||"";
  $("x_ca_file").value=e["ca-file"]||"";
  $("x_ca_path").value=e["ca-path"]||"";
  $("l_log_file").value=e["log-file"]||"";
  $("l_log_size").value=e["log-size"]||"";
  $("l_log_num").value=e["log-num"]||"";
  $("l_log_console").checked=e["log-console"]==="yes"||e["log-console"]==="";
  $("l_log_syslog").checked=e["log-syslog"]==="yes"||e["log-syslog"]==="";
  $("l_audit_file").value=e["audit-file"]||"";
  $("l_audit_size").value=e["audit-size"]||"";
  $("l_audit_num").value=e["audit-num"]||"";
  $("l_audit_console").checked=e["audit-console"]==="yes"||e["audit-console"]==="";
  $("a_force_qtype_soa").value=e["force-qtype-SOA"]||"";
  $("a_force_aaaa_soa").checked=e["force-AAAA-SOA"]==="yes"||e["force-AAAA-SOA"]==="";
  $("a_force_no_cname").checked=e["force-no-CNAME"]==="yes"||e["force-no-CNAME"]==="";
  $("a_ecs").value=e["edns-client-subnet"]||"";
  $("a_dns64").value=e["dns64"]||"";
  $("a_ddns_domain").value=e["ddns-domain"]||"";
  $("a_local_domain").value=e["local-domain"]||"";
  $("a_dnsmasq_lease").value=e["dnsmasq-lease-file"]||"";
}

function matchLine(t,k){if(!t)return"";for(const l of t.split("\n")){const x=l.trim();if(x.startsWith(k+" ")||x.startsWith(k+"\t"))return x.substring(k.length).trim();}return"";}

// ── Render ────────────────────────────────────────────────────────────
function renderAll(){
  renderUpstreams();autoSave(); renderSets();autoSave(); renderRouting();autoSave(); renderIpRules();autoSave(); renderConfFiles();autoSave();
}

// -- Upstreams (grouped + editable) --
const LABELS = {kind:"协议",value:"地址",groups:"分组",proxy_name:"代理",bootstrap_dns:"Bootstrap",host_ip:"Host IP",subnet:"Subnet",options:"额外参数",note:"备注"};
const KINDS = {server:"UDP","server-tcp":"TCP","server-tls":"TLS","server-https":"HTTPS","server-quic":"QUIC","server-http3":"HTTP3"};

function getProxyNames(){
  const names=[];
  const raw=S.extras["proxy-server"]||"";
  const m=raw.match(/-name\s+(\S+)/);
  if(m)names.push(m[1]);
  // Also check upstreams for any proxy_name used
  S.upstreams.forEach(u=>{if(u.proxy_name&&!names.includes(u.proxy_name))names.push(u.proxy_name);});
  return names;
}

// Parse composite options string into individual fields
function parseUpOpts(opts){
  const o={http_host:"",tls_sni:"",spki_pin:"",iface:"",set_mark:"",blacklist_ip:"",whitelist_ip:"",tcp_ka:"",tls_host_verify:false,no_check_cert:false,extra:""};
  if(!opts)return o;
  const tokens=opts.match(/(?:[^\s"]+|"[^"]*")+/g)||[];
  for(let i=0;i<tokens.length;i++){
    const t=tokens[i];
    if(t==="-http-host"&&tokens[i+1]){o.http_host=tokens[++i];}
    else if(t==="-host-name"&&tokens[i+1]){o.tls_sni=tokens[++i];}
    else if(t==="-spki-pin"&&tokens[i+1]){o.spki_pin=tokens[++i];}
    else if(t==="-interface"&&tokens[i+1]){o.iface=tokens[++i];}
    else if(t==="-set-mark"&&tokens[i+1]){o.set_mark=tokens[++i];}
    else if(t==="-blacklist-ip"&&tokens[i+1]){o.blacklist_ip=tokens[++i];}
    else if(t==="-whitelist-ip"&&tokens[i+1]){o.whitelist_ip=tokens[++i];}
    else if(t==="-tcp-keepalive"&&tokens[i+1]){o.tcp_ka=tokens[++i];}
    else if(t==="-tls-host-verify"){o.tls_host_verify=true;}
    else if(t==="-no-check-certificate"){o.no_check_cert=true;}
    else{o.extra+=(o.extra?" ":"")+t;}
  }
  return o;
}

function classifyUpstreams(){
  const groups={}, nongroup=[];
  S.upstreams.forEach((u,i)=>{
    const gs=(u.groups||[]).filter(Boolean);
    if(gs.length){
      gs.forEach(g=>{
        if(!groups[g])groups[g]=[];
        groups[g].push({...u,_idx:i});
      });
    }else{
      nongroup.push({...u,_idx:i});
    }
  });
  return {groups,nongroup};
}

function renderUpstreams(){
  const box=$("upstreamGroups");box.innerHTML="";
  const {groups,nongroup}=classifyUpstreams();
  const seen=new Set();
  const allGroups=Object.keys(groups).sort();
  if(!allGroups.length&&!nongroup.length){
    box.innerHTML=`<div class="card empty">还没有可展示的上游 DNS。若配置文件中已有内容，请先刷新；如果仍为空，说明后端尚未正确解析当前 upstream 配置。</div>`;
    return;
  }
  
  // Render grouped upstreams
  allGroups.forEach(gname=>{
    const items=groups[gname];
    const d=document.createElement("div");d.className="up-section";
    d.innerHTML=`<div class="up-sec-head">
      <div><input class="up-group-name-inline" value="${esc(gname)}" data-old="${esc(gname)}" style="font-weight:700;font-size:15px;border:none;background:transparent;padding:2px 4px;width:120px">
      <p class="dim">属于此组的上游 DNS。通过 nameserver /域名/${esc(gname)} 指定域名走这个组</p></div>
      <span class="bd">${items.length} 个上游</span></div>`;
    items.forEach(u=>{
      if(!seen.has(u._idx)){seen.add(u._idx);d.appendChild(upCard(u,allGroups));}
    });
    // Group name change handler
    d.querySelector(".up-group-name-inline").addEventListener("change",function(){
      const old=this.dataset.old,neu=this.value.trim();
      if(!neu||neu===old)return;
      S.upstreams.forEach(u=>{
        const idx=(u.groups||[]).indexOf(old);
        if(idx>=0)u.groups[idx]=neu;
      });
      renderUpstreams();autoSave();
    });
    box.appendChild(d);
  });
  
  // Render non-grouped
  if(nongroup.length){
    const d=document.createElement("div");d.className="up-section";
    d.innerHTML=`<div class="up-sec-head"><div><strong>默认组（未分组）</strong><p class="dim">没有指定分组的上游 DNS。所有未命中分流规则的域名都会使用这些上游</p></div><span class="bd">${nongroup.length} 个上游</span></div>`;
    nongroup.forEach(u=>d.appendChild(upCard(u,allGroups)));
    box.appendChild(d);
  }
  
  if(!allGroups.length&&!nongroup.length)box.innerHTML=`<div class="card empty">还没有上游 DNS。在下方新增。</div>`;
}

function upCard(u,allGroups){
  const card=document.createElement("div");card.className="card up-card";
  const gs=(u.groups||[])[0]||"";
  const proto=KINDS[u.kind]||u.kind||"server";
  const saved={kind:u.kind,value:u.value,group:gs,proxy_name:u.proxy_name||"",bootstrap_dns:u.bootstrap_dns||"",host_ip:u.host_ip||"",subnet:u.subnet||"",fallback:u.fallback,exclude_default_group:u.exclude_default_group,options:u.options||"",note:u.note||""};
  const opts=parseUpOpts(u.options||"");

  // Build group options for select
  const gOpts=allGroups.map(g=>`<option value="${esc(g)}" ${saved.group===g?'selected':''}>${esc(g)}</option>`).join("");
  const otherSel=saved.group&&!allGroups.includes(saved.group)?`<option value="${esc(saved.group)}" selected>${esc(saved.group)}</option>`:"";
  // Proxy select options
  const pxyNames=getProxyNames();
  const pxyOpts=pxyNames.map(p=>`<option value="${esc(p)}" ${saved.proxy_name===p?'selected':''}>${esc(p)}</option>`).join("");
  const pxyOther=saved.proxy_name&&!pxyNames.includes(saved.proxy_name)?`<option value="${esc(saved.proxy_name)}" selected>${esc(saved.proxy_name)}</option>`:"";

  card.innerHTML=`
    <div class="up-summary">
      <div class="up-info">
        <span class="bd">${esc(proto)}</span>
        <strong>${esc(u.value||"")}</strong>
        <span class="dim">${esc(gs||"默认组")}</span>
        ${u.fallback?'<span class="bd w">fallback</span>':''}
        ${u.proxy_name?`<span class="bd">proxy:${esc(u.proxy_name)}</span>`:''}
        ${u.host_ip?`<span class="dim">→ ${esc(u.host_ip)}</span>`:''}
      </div>
      <div class="up-acts">
        <button class="sec up-edit-btn">编辑</button>
        <button class="ghost up-del" data-idx="${u._idx}">删除</button>
      </div>
    </div>
    <div class="up-edit hidden">
      <div class="fg fg4">
        <div class="field"><label>协议</label><select class="ue-kind">${Object.entries(KINDS).map(([k,v])=>`<option value="${k}" ${saved.kind===k?'selected':''}>${v}</option>`).join("")}</select></div>
        <div class="field s2"><label>地址</label><input class="ue-value" value="${esc(saved.value)}"></div>
        <div class="field"><label>分组</label><select class="ue-groups">${otherSel}${gOpts}<option value="">（新建分组）</option></select><input class="ue-group-new hidden" placeholder="输入新分组名" style="margin-top:4px"></div>
        <div class="field"><label>代理</label><select class="ue-proxy"><option value="" ${!saved.proxy_name?'selected':''}>不使用代理</option>${pxyOpts}${pxyOther}</select></div>
        <div class="field"><label>Bootstrap DNS</label><input class="ue-bootstrap" value="${esc(saved.bootstrap_dns)}"></div>
        <div class="field"><label>Host IP</label><input class="ue-hostip" value="${esc(saved.host_ip)}"></div>
        <div class="field"><label>Subnet</label><input class="ue-subnet" value="${esc(saved.subnet)}"></div>
        <div class="field"><label>HTTP Host 头</label><input class="ue-http-host" value="${esc(opts.http_host)}"></div>
        <div class="field"><label>TLS SNI</label><input class="ue-tls-sni" value="${esc(opts.tls_sni)}"></div>
        <div class="field"><label>SPKI Pin</label><input class="ue-spki-pin" value="${esc(opts.spki_pin)}"></div>
        <div class="field"><label>绑定网口</label><input class="ue-iface" value="${esc(opts.iface)}"></div>
        <div class="field"><label>SO_MARK</label><input class="ue-set-mark" value="${esc(opts.set_mark)}"></div>
        <div class="field"><label>黑名单 IP</label><input class="ue-blacklist-ip" value="${esc(opts.blacklist_ip)}"></div>
        <div class="field"><label>白名单 IP</label><input class="ue-whitelist-ip" value="${esc(opts.whitelist_ip)}"></div>
        <div class="field"><label>TCP Keepalive (ms)</label><input class="ue-tcp-ka" value="${esc(opts.tcp_ka)}"></div>
        <div class="field"><label>额外参数</label><input class="ue-opts" value="${esc(opts.extra)}"></div>
        <div class="field"><label>备注</label><input class="ue-note" value="${esc(saved.note)}"></div>
        <label class="ci"><input type="checkbox" class="ue-exclude" ${saved.exclude_default_group?'checked':''}> 排除默认组</label>
        <label class="ci"><input type="checkbox" class="ue-fallback" ${saved.fallback?'checked':''}> Fallback 兜底</label>
        <label class="ci"><input type="checkbox" class="ue-tls-host-verify" ${opts.tls_host_verify?'checked':''}> 验证 TLS 主机名</label>
        <label class="ci"><input type="checkbox" class="ue-no-check-cert" ${opts.no_check_cert?'checked':''}> 跳过证书验证</label>
      </div>
      <button class="sec up-save-btn">保存修改</button>
      <button class="ghost up-cancel-btn">取消</button>
    </div>`;

  // Group select: show "new group" input when empty selected
  const gsSelect=card.querySelector(".ue-groups");
  const gsNew=card.querySelector(".ue-group-new");
  gsSelect.addEventListener("change",()=>{
    if(gsSelect.value===""){gsNew.classList.remove("hidden");gsNew.focus();}
    else gsNew.classList.add("hidden");
  });

  card.querySelector(".up-edit-btn").onclick=()=>{card.querySelector(".up-edit").classList.toggle("hidden");};
  card.querySelector(".up-save-btn").onclick=()=>saveUpEdit(u._idx,card);
  card.querySelector(".up-cancel-btn").onclick=()=>{card.querySelector(".up-edit").classList.add("hidden");};
  card.querySelector(".up-del").onclick=()=>{S.upstreams.splice(u._idx,1);renderUpstreams();autoSave();};
  return card;
}

function saveUpEdit(idx,card){
  const e=card.querySelector(".up-edit");
  const gsSelect=e.querySelector(".ue-groups");
  let group=gsSelect.value;
  if(group===""){
    // New group entered
    group=e.querySelector(".ue-group-new").value.trim();
  }
  S.upstreams[idx]={
    kind:e.querySelector(".ue-kind").value,
    value:e.querySelector(".ue-value").value.trim(),
    groups:group?[group]:[],
    exclude_default_group:e.querySelector(".ue-exclude").checked,
    proxy_name:e.querySelector(".ue-proxy").value,
    bootstrap_dns:e.querySelector(".ue-bootstrap").value.trim(),
    host_ip:e.querySelector(".ue-hostip").value.trim(),
    subnet:e.querySelector(".ue-subnet").value.trim(),
    fallback:e.querySelector(".ue-fallback").checked,
    options:[
      e.querySelector(".ue-http-host").value.trim()?"-http-host "+e.querySelector(".ue-http-host").value.trim():"",
      e.querySelector(".ue-tls-sni").value.trim()?"-host-name "+e.querySelector(".ue-tls-sni").value.trim():"",
      e.querySelector(".ue-spki-pin").value.trim()?"-spki-pin "+e.querySelector(".ue-spki-pin").value.trim():"",
      e.querySelector(".ue-iface").value.trim()?"-interface "+e.querySelector(".ue-iface").value.trim():"",
      e.querySelector(".ue-set-mark").value.trim()?"-set-mark "+e.querySelector(".ue-set-mark").value.trim():"",
      e.querySelector(".ue-blacklist-ip").value.trim()?"-blacklist-ip "+e.querySelector(".ue-blacklist-ip").value.trim():"",
      e.querySelector(".ue-whitelist-ip").value.trim()?"-whitelist-ip "+e.querySelector(".ue-whitelist-ip").value.trim():"",
      e.querySelector(".ue-tcp-ka").value.trim()?"-tcp-keepalive "+e.querySelector(".ue-tcp-ka").value.trim():"",
      e.querySelector(".ue-tls-host-verify").checked?"-tls-host-verify":"",
      e.querySelector(".ue-no-check-cert").checked?"-no-check-certificate":"",
      e.querySelector(".ue-opts").value.trim(),
      ].filter(Boolean).join(" "),
    note:e.querySelector(".ue-note").value.trim(),
  };
  renderUpstreams();autoSave();
}

function addUpstream(){
  const gsSel=$("upGroupsSelect");
  let group=gsSel?gsSel.value:"";
  if(group==="_new_")group=$("upGroupNew").value.trim();
  S.upstreams.push({
    kind:$("upProto").value, value:$("upValue").value.trim(),
    groups:group?[group]:[],
    exclude_default_group:$("upExcludeDefault").checked, proxy_name:$("upProxy").value.trim(),
    bootstrap_dns:$("upBootstrap").value.trim(), host_ip:$("upHostIp").value.trim(),
    subnet:$("upSubnet").value.trim(), fallback:$("upFallback").checked,
    options:[
      $("upHttpHost").value.trim()?"-http-host "+$("upHttpHost").value.trim():"",
      $("upTlsSni").value.trim()?"-host-name "+$("upTlsSni").value.trim():"",
      $("upSpkiPin").value.trim()?"-spki-pin "+$("upSpkiPin").value.trim():"",
      $("upInterface").value.trim()?"-interface "+$("upInterface").value.trim():"",
      $("upSetMark").value.trim()?"-set-mark "+$("upSetMark").value.trim():"",
      $("upBlacklistIp").value.trim()?"-blacklist-ip "+$("upBlacklistIp").value.trim():"",
      $("upWhitelistIp").value.trim()?"-whitelist-ip "+$("upWhitelistIp").value.trim():"",
      $("upTcpKa").value.trim()?"-tcp-keepalive "+$("upTcpKa").value.trim():"",
      $("upTlsHostVerify").checked?"-tls-host-verify":"",
      $("upNoCheckCert").checked?"-no-check-certificate":"",
      ].filter(Boolean).join(" "),
    note:$("upNote").value.trim(),
  });
  ["upValue","upProxy","upBootstrap","upHostIp","upSubnet","upHttpHost","upTlsSni","upSpkiPin","upInterface","upSetMark","upBlacklistIp","upWhitelistIp","upTcpKa","upNote"].forEach(id=>$(id).value="");
  if($("upGroupNew"))$("upGroupNew").value="";
  $("upExcludeDefault").checked=false;$("upFallback").checked=false;$("upTlsHostVerify").checked=false;$("upNoCheckCert").checked=false;
  renderUpstreams();autoSave();refreshGroupSelects();
}

function refreshGroupSelects(){
  const names=new Set();
  S.upstreams.forEach(u=>{(u.groups||[]).forEach(g=>{if(g)names.add(g);});});
  const opts=[...names].sort().map(g=>`<option value="${esc(g)}">${esc(g)}</option>`).join("");
  const xtra=`<option value="_new_">（新建分组…）</option>`;
  const sel=$("upGroupsSelect");
  if(sel){sel.innerHTML=opts+xtra;sel.onchange=function(){const n=$("upGroupNew");if(this.value==="_new_"){n.classList.remove("hidden");n.focus();}else n.classList.add("hidden");};}
  // Populate proxy select in add form
  const pxy=$("upProxy");
  if(pxy){
    const pn=getProxyNames();
    pxy.innerHTML=`<option value="">不使用代理</option>`+pn.map(p=>`<option value="${esc(p)}">${esc(p)}</option>`).join("");
  }
}

// -- Sets (domain-set + ip-set unified) --
function renderSets(){
  const box=$("domainSetList");box.innerHTML="";
  // Merge existing assets from main config + managed assets
  const seen=new Set();
  const all=[];
  // Managed domain-sets first
  (S.managed.domain_sets||[]).forEach(ds=>{seen.add(ds.name);all.push({...ds,source:"托管"});});
  // Existing domain-sets from main config
  (S.config&&S.config.rule_set_assets||[]).forEach(a=>{
    if(!seen.has(a.name)){seen.add(a.name);all.push({name:a.name,file:a.file||"",set_type:a.set_type||"list",source:a.source||"主配置"});}
  });
  if(!all.length){box.innerHTML=`<div class="card empty">还没有域名集合。在下方新增。</div>`;return;}
  all.forEach((ds,i)=>{
    const div=document.createElement("div");div.className="card ds-row";
    div.innerHTML=`
      <div class="ds-fields">
        <div class="field"><label>名称</label><input class="ds-name" value="${esc(ds.name)}"></div>
        <div class="field"><label>类型</label><input class="ds-type" value="${esc(ds.set_type||'list')}"></div>
        <div class="field ds-file-field"><label>文件路径</label><input class="ds-file" value="${esc(ds.file||'')}"></div>
      </div>
      <button class="ghost ds-del" data-name="${esc(ds.name)}">删除</button>
    `;
    // Bind save on change
    ["ds-name","ds-type","ds-file"].forEach(cls=>{
      div.querySelector(`.${cls}`).addEventListener("change",()=>syncDomainSets(all,div));
    });
    div.querySelector(".ds-del").onclick=()=>{
      // Remove from both managed and the all list
      const name=ds.name;
      S.managed.domain_sets=(S.managed.domain_sets||[]).filter(x=>x.name!==name);
      all.splice(i,1);
      renderDomainSets();
    };
    box.appendChild(div);
  });
  // Save all button
  const saveDiv=document.createElement("div");
  saveDiv.innerHTML=`<button class="pri ds-save-all">保存域名集合到托管文件</button><span class="dim" style="margin-left:10px;font-size:12px">原 smartdns.conf 中的 domain-set 定义请手动移除，避免重复</span>`;
  saveDiv.querySelector(".ds-save-all").onclick=()=>{syncAllDomainSets(all);renderDomainSets();};
  box.appendChild(saveDiv);
}

function syncDomainSets(all, rowDiv){
  // Read all rows and update managed array
  const rows=document.querySelectorAll(".ds-row");
  const result=[];
  rows.forEach(r=>{
    const n=r.querySelector(".ds-name")?.value?.trim();
    const t=r.querySelector(".ds-type")?.value?.trim()||"list";
    const f=r.querySelector(".ds-file")?.value?.trim();
    if(n&&f)result.push({name:n,file:f,set_type:t});
  });
  S.managed.domain_sets=result;
}

function syncAllDomainSets(all){
  const rows=document.querySelectorAll(".ds-row");
  const result=[];
  rows.forEach(r=>{
    const n=r.querySelector(".ds-name")?.value?.trim();
    const t=r.querySelector(".ds-type")?.value?.trim()||"list";
    const f=r.querySelector(".ds-file")?.value?.trim();
    if(n&&f)result.push({name:n,file:f,set_type:t});
  });
  S.managed.domain_sets=result;
}

function addDomainSet(){
  const n=$("dsName").value.trim(),f=$("dsFile").value.trim();
  if(!n||!f){alert("名称和文件路径不能为空");return;}
  S.managed.domain_sets.push({name:n,file:f,set_type:$("dsType").value});
  $("dsName").value="";$("dsFile").value="";
  renderSets();autoSave();
}

// -- Sets (domain-set + ip-set unified) --
function renderSets(){
  const box=$("setList");if(!box)return;box.innerHTML="";
  const all=[];
  const seen=new Set();
  (S.managed.domain_sets||[]).forEach(ds=>{seen.add(ds.name);all.push({type:"domain-set",name:ds.name,file:ds.file});});
  (S.config&&S.config.rule_set_assets||[]).forEach(a=>{if(!seen.has(a.name)){seen.add(a.name);all.push({type:"domain-set",name:a.name,file:a.file||""});}});
  (S.managed.ip_sets||[]).forEach(is=>{all.push({type:"ip-set",name:is.name,file:is.file});});
  if(!all.length){box.innerHTML=`<div class="card empty">还没有集合。在上方新增。</div>`;return;}
  all.forEach((s,i)=>{
    const d=document.createElement("div");d.className="card rc";
    d.innerHTML=`<div class="rm"><span class="bd">${s.type==="domain-set"?"域名集合":"IP 集合"}</span> <strong>${esc(s.name)}</strong> <code>${esc(s.file||"")}</code></div><button class="ghost set-del" data-idx="${i}">删除</button>`;
    d.querySelector(".set-del").onclick=()=>{all.splice(i,1);syncSets(all);};
    box.appendChild(d);
  });
}
function syncSets(all){
  S.managed.domain_sets=[];S.managed.ip_sets=[];
  (all||[]).forEach(s=>{if(s.type==="domain-set")S.managed.domain_sets.push({name:s.name,file:s.file,set_type:"list"});else S.managed.ip_sets.push({name:s.name,file:s.file});});
}
function addSet(){
  const t=$("setType").value,n=$("setName").value.trim(),f=$("setFile").value.trim();
  if(!n||!f){alert("名称和文件路径不能为空");return;}
  if(t==="domain-set")S.managed.domain_sets.push({name:n,file:f,set_type:"list"});
  else S.managed.ip_sets.push({name:n,file:f});
  $("setName").value="";$("setFile").value="";renderSets();autoSave();
}

// -- Routing (unified: detected + managed, all editable) --
const RT_LABEL={nameserver:"上游分流","domain-rules":"域名规则",address:"地址改写",cname:"CNAME"};
const RT_ICON={nameserver:"🔀","domain-rules":"⚙️",address:"📌",cname:"🔗"};

function describeRouting(r){
  const t=r.target||"", v=r.value||"";
  if(r.rule_type==="nameserver"){
    const ds=t.match(/domain-set:(\w+)/);
    const g=v||"默认";
    if(ds)return `域名集合 <span class="clr-ds">${ds[1]}</span> → <span class="clr-grp">${g}</span> 组`;
    const plain=t.replace(/^\//,"").replace(/\/$/,"");
    if(t.startsWith("/") && plain && !plain.includes("."))return `后缀 <span class="clr-dom">.${plain}</span> → <span class="clr-grp">${g}</span> 组`;
    return `<span class="clr-dom">${plain}</span> → <span class="clr-grp">${g}</span> 组`;
  }
  if(r.rule_type==="domain-rules"){
    const ds=t.match(/domain-set:(\w+)/);
    const prefix=ds?`域名集合 <b>${ds[1]}</b>`:`<b>${t.replace(/^\//,"").replace(/\/$/,"")}</b>`;
    const labels=[];
    if(v.includes("-no-cache"))labels.push("不缓存");
    if(v.includes("-no-serve-expired"))labels.push("不兜底过期");
    if(v.includes("-no-ip-alias"))labels.push("忽略IP别名");
    if(v.includes("-address #6"))labels.push("禁IPv6");
    if(v.includes("-address #4"))labels.push("禁IPv4");
    const ns=v.match(/-nameserver\s+(\S+)/);if(ns)labels.push(`上游组:${ns[1]}`);
    if(v.includes("-dualstack-ip-selection"))labels.push("双栈优选");
    if(labels.length)return `${prefix}：${labels.join("、")}`;
    return `${prefix} 应用规则：<code>${esc(v)}</code>`;
  }
  if(r.rule_type==="address"){
    const ds=t.match(/domain-set:(\w+)/);
    const prefix=ds?`域名集合 <b>${ds[1]}</b>`:`<b>${t.replace(/^\//,"")}</b>`;
    if(v==="#"||v==="#4"||v==="#6")return `${prefix} 屏蔽（返回空地址）`;
    if(v.startsWith("#"))return `${prefix} 屏蔽（${v}）`;
    return `${prefix} 固定返回 <b>${v||"空"}</b>`;
  }
  if(r.rule_type==="cname"){
    return `<b>${t.replace(/^\//,"")}</b> 别名到 <b>${v}</b>`;
  }
  return `${t} → ${v}`;
}

function routingKey(rule){
  return `${rule.rule_type}|${rule.target}|${rule.value||""}`;
}

function deletedRoutingKeys(){
  return new Set(S.managed._deleted||[]);
}

function markRoutingDeleted(rule){
  const key=routingKey(rule);
  if(!S.managed._deleted)S.managed._deleted=[];
  if(!S.managed._deleted.includes(key))S.managed._deleted.push(key);
}

function unmarkRoutingDeleted(rule){
  const key=routingKey(rule);
  S.managed._deleted=(S.managed._deleted||[]).filter(x=>x!==key);
}

function mergedRoutingItems(types){
  const result=[], seen=new Set(), deleted=deletedRoutingKeys();
  (S.managed.routing_items||[]).forEach(r=>{
    if(types.includes(r.rule_type)){
      const key=routingKey(r);
      seen.add(key);
      result.push({...r,_src:"托管"});
    }
  });
  (S.config&&S.config.routing_items||[]).forEach(r=>{
    if(types.includes(r.rule_type)){
      const key=routingKey(r);
      if(!seen.has(key)&&!deleted.has(key)){
        seen.add(key);
        result.push({...r,_src:"主配置"});
      }
    }
  });
  return result;
}

function removeRoutingRule(rule){
  const key=routingKey(rule);
  S.managed.routing_items=(S.managed.routing_items||[]).filter(x=>routingKey(x)!==key);
  if(rule._src==="主配置")markRoutingDeleted(rule);
}

function allRoutingItems(){
  return mergedRoutingItems(["nameserver"]);
}

function renderRouting(){
  const box=$("routingList");box.innerHTML="";
  const items=allRoutingItems();
  if(!items.length){box.innerHTML=`<div class="card empty">还没有分流规则。在上方新增。</div>`;return;}
  // Group by type
  const byType={};
  items.forEach(r=>{if(!byType[r.rule_type])byType[r.rule_type]=[];byType[r.rule_type].push(r);});
  ["nameserver"].forEach(type=>{
    const list=byType[type];if(!list||!list.length)return;
    const sec=document.createElement("div");sec.className="rt-section";
    sec.innerHTML=`<div class="rt-sec-head"><strong>${RT_ICON[type]||""} ${RT_LABEL[type]||type}</strong><span class="bd">${list.length} 条</span></div>`;
    list.forEach((r,i)=>{
	      const card=document.createElement("div");card.className="card rt-card";
	      const desc=describeRouting(r);
	      card.innerHTML=`
	        <div class="rt-summary">
	          <span class="drag-handle" draggable="true">⋮⋮</span>
	          <div class="rt-desc">${desc}</div>
	          <div class="rt-acts">
	            <button class="ghost rt-up-btn" data-idx="${i}" data-type="${type}">↑</button>
	            <button class="ghost rt-down-btn" data-idx="${i}" data-type="${type}">↓</button>
	            <button class="sec rt-edit-btn">编辑</button>
	            <button class="ghost rt-del-btn">删除</button>
	          </div>
	        </div>
	        <div class="rt-edit hidden">
	          <div class="fg">
	            <div class="field"><label>类型</label><select class="rte-type">${Object.entries(RT_LABEL).map(([k,v])=>`<option value="${k}" ${r.rule_type===k?'selected':''}>${v}</option>`).join("")}</select></div>
	            <div class="field s2"><label>匹配目标</label><input class="rte-target" value="${esc(r.target)}"></div>
	            <div class="field s2"><label>参数</label><input class="rte-value" value="${esc(r.value||'')}"></div>
	          </div>
	          <button class="sec rte-save-btn">保存</button>
	          <button class="ghost rte-cancel-btn">取消</button>
	        </div>`;
	      // --- Event handlers ---
	      card.querySelector(".rt-edit-btn").onclick=()=>card.querySelector(".rt-edit").classList.toggle("hidden");
	      card.querySelector(".rte-cancel-btn").onclick=()=>card.querySelector(".rt-edit").classList.add("hidden");
	      card.querySelector(".rte-save-btn").onclick=()=>{
	        const e=card.querySelector(".rt-edit");
	        r.rule_type=e.querySelector(".rte-type").value;
	        r.target=e.querySelector(".rte-target").value.trim();
	        r.value=e.querySelector(".rte-value").value.trim();
	        renderRouting();autoSave();
	      };
      // Delete - with visibility
      const delBtn=card.querySelector(".rt-del-btn");
      if(!delBtn){alert("找不到删除按钮元素! card:"+card.innerHTML.substring(0,100));return;}
      delBtn.onclick=()=>{
        removeRoutingRule(r);
        renderRouting();autoSave();
      };
	      // Drag and drop
	      const dh=card.querySelector(".drag-handle");
	      if(dh){dh.addEventListener("dragstart",e=>{e.dataTransfer.setData("text/plain",JSON.stringify({idx:i,type:type}));card.style.opacity="0.4";});
	      dh.addEventListener("dragend",()=>{card.style.opacity="1";});}
	      card.addEventListener("dragover",e=>{e.preventDefault();});
	      card.addEventListener("drop",e=>{e.preventDefault();e.stopPropagation();
	        try{const d=JSON.parse(e.dataTransfer.getData("text/plain"));if(d.idx===i)return;
	        const lst=byType[d.type];if(!lst)return;
	        const [mv]=lst.splice(d.idx,1);lst.splice(i,0,mv);
	        S.managed.routing_items=[];
	        ["nameserver"].forEach(t=>{(byType[t]||[]).forEach(x=>S.managed.routing_items.push({rule_type:t,target:x.target,value:x.value||""}));});
	        renderRouting();autoSave();}catch(ex){}
	      });
      sec.appendChild(card);
    });
    // Add move buttons
    if(list.length>1){
      const bar=document.createElement("div");bar.style.cssText="display:flex;gap:4px;margin-top:4px";
      bar.innerHTML=`<span class="dim" style="font-size:11px">⚠️ 匹配从上到下，点击 ↑↓ 箭头调整顺序</span>`;
      sec.appendChild(bar);
    }
    box.appendChild(sec);
  });
  // Bind move buttons
  document.querySelectorAll(".rt-up-btn,.rt-down-btn").forEach(b=>{
    b.onclick=()=>{
      const idx=parseInt(b.dataset.idx),type=b.dataset.type;
      const list=byType[type];if(!list)return;
      if(b.classList.contains("rt-up-btn")&&idx>0){[list[idx-1],list[idx]]=[list[idx],list[idx-1]];}
      else if(b.classList.contains("rt-down-btn")&&idx<list.length-1){[list[idx],list[idx+1]]=[list[idx+1],list[idx]];}
      syncRoutingFromList(byType);
      renderRouting();
    };
  });
}

function syncRoutingFromList(byType){
  const result=[];
  ["nameserver"].forEach(type=>{(byType[type]||[]).forEach(r=>{result.push({rule_type:type,target:r.target,value:r.value||""});});});
  const other=(S.managed.routing_items||[]).filter(r=>r.rule_type!=="nameserver");
  S.managed.routing_items=[...result,...other];
  autoSave();
}
function syncRoutingFromUI(){
  const result=[];
  document.querySelectorAll(".rt-card").forEach(card=>{
    const type=card.querySelector(".rte-type")?.value;
    const target=card.querySelector(".rte-target")?.value?.trim();
    const value=card.querySelector(".rte-value")?.value?.trim();
    if(type&&target)result.push({rule_type:type,target:target,value:value||""});
  });
  S.managed.routing_items=result;
}

function addRouting(){
  const type="nameserver", tt=$("rtTargetType").value;
  let target,value="";
  if(tt==="domain-set"){
    const n=$("rtTargetDS").value;if(!n){alert("请选择域名集合");return;}
    target="/domain-set:"+n;
  }else{
    let raw=$("rtTargetInput").value.trim();if(!raw){alert("请输入目标");return;}
    // Strip leading dot for suffix (用户输入 .cn → cn)
    if(tt==="suffix")raw=raw.replace(/^\./,"");
    target="/"+raw.replace(/^\/+|\/+$/g,"");
  }
  const g=$("rtValSelect").value;if(!g){alert("请选择上游组");return;}
  value=g;
  const item={rule_type:type,target:target,value:value};
  const current=allRoutingItems().map(r=>({rule_type:r.rule_type,target:r.target,value:r.value||""}));
  current.push(item);
  const other=(S.managed.routing_items||[]).filter(r=>r.rule_type!=="nameserver");
  S.managed.routing_items=[...current,...other];
  unmarkRoutingDeleted(item);
  if(tt!=="domain-set")$("rtTargetInput").value="";
  renderRouting();autoSave();
}


function populateRoutingDropdowns(){
  const ds=[];
  (S.managed.domain_sets||[]).forEach(d=>ds.push(d.name));
  (S.config&&S.config.rule_set_assets||[]).forEach(a=>{if(!ds.includes(a.name))ds.push(a.name);});
  const dsOpts=ds.map(n=>`<option value="${n}">${n}</option>`).join("");
  $("rtTargetDS").innerHTML=dsOpts;
  $("acTargetDS").innerHTML=dsOpts;
  $("drTargetDS").innerHTML=dsOpts;
  const grps=new Set();
  S.upstreams.forEach(u=>{(u.groups||[]).forEach(g=>{if(g)grps.add(g);});});
  const gOpts=[...grps].sort().map(g=>`<option value="${g}">${g}</option>`).join("");
  $("rtValSelect").innerHTML=gOpts;
  $("drNSGroup").innerHTML='<option value="">不指定</option>'+gOpts;
  // Target type switches
  $("acTargetType").onchange=function(){$("acTargetDS").style.display=this.value==="domain-set"?"":"none";$("acTargetInput").style.display=this.value!=="domain-set"?"":"none";};
  $("acTargetType").onchange();
  $("drTargetType").onchange=function(){$("drTargetDS").style.display=this.value==="domain-set"?"":"none";$("drTargetInput").style.display=this.value!=="domain-set"?"":"none";};
  $("drTargetType").onchange();
}

// Show/hide fields based on rule type and target type
function onRtFormChange(){
  const tt=$("rtTargetType").value;
  $("rtTargetDS").style.display=tt==="domain-set"?"":"none";
  $("rtTargetInput").style.display=tt!=="domain-set"?"":"none";
  if(tt==="suffix")$("rtTargetInput").placeholder="cn 或 com";
  else $("rtTargetInput").placeholder="example.com";
}
$("rtTargetType").addEventListener("change",onRtFormChange);


// -- Groups --
function renderGroups(){const b=$("groupList");b.innerHTML="";const it=S.managed.rule_groups||[];if(!it.length){b.innerHTML=`<div class="card empty">还没有规则组。</div>`;return;}it.forEach((x,i)=>{const d=document.createElement("div");d.className="card rc";const ms=(x.matchers||[]).map(m=>`<li>${esc(m)}</li>`).join("")||"<li>无匹配条件</li>";d.innerHTML=`<div class="rm"><span class="bd">group</span> <strong>${esc(x.name)}</strong> <span class="dim">继承: ${esc(x.inherit||"默认")}</span><ul class="il">${ms}</ul></div><button class="ghost del-rg" data-idx="${i}">删除</button>`;b.appendChild(d);});document.querySelectorAll(".del-rg").forEach(b=>{b.onclick=()=>{S.managed.rule_groups.splice(parseInt(b.dataset.idx),1);renderGroups();autoSave();};});}
function addGroup(){const n=$("rgName").value.trim();if(!n){alert("组名不能为空");return;}S.managed.rule_groups.push({name:n,inherit:$("rgInherit").value.trim(),matchers:$("rgMatchers").value.split("\n").map(x=>x.trim()).filter(Boolean)});$("rgName").value="";$("rgInherit").value="";$("rgMatchers").value="";renderGroups();autoSave();}


// -- Address & CNAME --
function renderAddrCname(){
  const box=$("addrCnameList");if(!box)return;box.innerHTML="";
  const items=mergedRoutingItems(["address","cname"]);
  if(!items.length){box.innerHTML=`<div class="card empty">还没有 address/cname 规则。</div>`;return;}
  items.forEach((r)=>{
    const d=document.createElement("div");d.className="card rc";
    const label=r.rule_type==="address"?"address":"cname";
    const ds=r.target.match(/domain-set:(\w+)/);
    const prefix=ds?`集合 <b>${ds[1]}</b>`:`<b>${r.target.replace(/^\//,"").replace(/\/$/,"")}</b>`;
    const v=r.value||"";
    const desc=r.rule_type==="address"?(v==="#"?"屏蔽":`返回 ${v}`):`别名→ ${v}`;
    d.innerHTML=`<div class="rm"><span class="bd">${esc(label)}</span> ${prefix}：${esc(desc)}</div><button class="ghost ac-del">删除</button>`;
    d.querySelector(".ac-del").onclick=()=>{removeRoutingRule(r);renderAddrCname();autoSave();};
    box.appendChild(d);
  });
}
function syncAddrCname(items){
  const other=(S.managed.routing_items||[]).filter(r=>r.rule_type!=="address"&&r.rule_type!=="cname");
  S.managed.routing_items=[...other,...items];
}
function addAddrCname(){
  const type=$("acType").value, tt=$("acTargetType").value;let target,label;
  if(tt==="domain-set"){const n=$("acTargetDS").value;if(!n){alert("请选择域名集合");return;}target="/domain-set:"+n;label=`集合 <b>${n}</b>`;}
  else{let raw=$("acTargetInput").value.trim();if(!raw){alert("请输入目标");return;}target="/"+raw.replace(/^\//,"")+"/";label=`<b>${raw}</b>`;}
  const val=$("acValInput").value.trim();
  S.managed.routing_items=S.managed.routing_items||[];
  const item={rule_type:type,target:target,value:val};
  S.managed.routing_items.push(item);
  unmarkRoutingDeleted(item);
  $("acTargetInput").value="";$("acValInput").value="";
  renderAddrCname();
  autoSave();
}

// -- Domain Rules (separate page) --
function renderDomainRules(){
  const box=$("domainRulesList");if(!box)return;box.innerHTML="";
  const items=mergedRoutingItems(["domain-rules"]);
  if(!items.length){box.innerHTML=`<div class="card empty">还没有域名规则。</div>`;return;}
  items.forEach((r)=>{
    const d=document.createElement("div");d.className="card rc";
    const ds=r.target.match(/domain-set:(\w+)/);
    const prefix=ds?`集合 <b>${ds[1]}</b>`:`<b>${r.target.replace(/^\//,"").replace(/\/$/,"")}</b>`;
    const labels=[];
    const v=r.value||"";
    if(v.includes("-no-cache"))labels.push("不缓存");
    if(v.includes("-no-serve-expired"))labels.push("不兜底过期");
    if(v.includes("-no-ip-alias"))labels.push("忽略IP别名");
    if(v.includes("-address #6"))labels.push("禁IPv6");
    if(v.includes("-address #4"))labels.push("禁IPv4");
    const ns=v.match(/-nameserver\s+(\S+)/);if(ns)labels.push("组:"+ns[1]);
    if(labels.length)r._desc=labels.join("、");else r._desc=v||"(无规则)";
    d.innerHTML=`<div class="rm"><span class="bd">domain-rules</span> ${prefix}：${esc(r._desc)}</div><button class="ghost dr-del">删除</button>`;
    d.querySelector(".dr-del").onclick=()=>{removeRoutingRule(r);renderDomainRules();autoSave();};
    box.appendChild(d);
  });
}
function syncDomainRules(items){
  const other=(S.managed.routing_items||[]).filter(r=>r.rule_type!=="domain-rules");
  S.managed.routing_items=[...other,...items.filter(r=>r.rule_type==="domain-rules")];
}
function addDomainRule(){
  const tt=$("drTargetType").value;let target;
  if(tt==="domain-set"){
    const n=$("drTargetDS").value;if(!n){alert("请选择域名集合");return;}
    target="/domain-set:"+n;
  }else{
    let raw=$("drTargetInput").value.trim();if(!raw){alert("请输入目标");return;}
    target="/"+raw.replace(/^\//,"")+"/";
  }
  const parts=[];
  if($("drNoCache").checked)parts.push("-no-cache");
  if($("drNoExpired").checked)parts.push("-no-serve-expired");
  if($("drNoIpAlias").checked)parts.push("-no-ip-alias");
  const a6=$("drAddr6").value;if(a6)parts.push(a6);
  const nsg=$("drNSGroup").value;if(nsg)parts.push("-nameserver "+nsg);
  const dl=$("drDual").value;if(dl)parts.push(dl);
  const rm=$("drResMode").value;if(rm)parts.push(rm);
  const tmin=$("drTtlMin").value.trim();if(tmin)parts.push("-rr-ttl-min "+tmin);
  const tmax=$("drTtlMax").value.trim();if(tmax)parts.push("-rr-ttl-max "+tmax);
  S.managed.routing_items=S.managed.routing_items||[];
  const item={rule_type:"domain-rules",target:target,value:parts.join(" ")};
  S.managed.routing_items.push(item);
  unmarkRoutingDeleted(item);
  // Build human labels BEFORE clearing form
  const human=[];
  if($("drNoCache").checked)human.push("不缓存");
  if($("drNoExpired").checked)human.push("不兜底过期");
  if($("drNoIpAlias").checked)human.push("忽略IP别名");
  if($("drAddr6").value==="-address #6")human.push("禁IPv6");
  if($("drAddr6").value==="-address #4")human.push("禁IPv4");
  const nsg2=$("drNSGroup").value;if(nsg2)human.push("组:"+nsg2);
  const dl2=$("drDual").value;if(dl2)human.push("双栈优选");
  // Clear form
  $("drTargetInput").value="";$("drNoCache").checked=false;$("drNoExpired").checked=false;$("drNoIpAlias").checked=false;
  $("drAddr6").value="";$("drNSGroup").value="";$("drDual").value="";$("drResMode").value="";$("drTtlMin").value="";$("drTtlMax").value="";
  // Show
  renderDomainRules();
  autoSave();
}


// -- Clients --
function renderClients(){const b=$("clientList");b.innerHTML="";const it=S.managed.client_rules||[];if(!it.length){b.innerHTML=`<div class="card empty">还没有客户端规则。</div>`;return;}it.forEach((x,i)=>{const d=document.createElement("div");d.className="card rc";d.innerHTML=`<div class="rm"><span class="bd">client</span> <strong>${esc(x.matcher)}</strong> <span class="dim">规则组: ${esc(x.group_name||"全局")}</span></div><button class="ghost del-cl" data-idx="${i}">删除</button>`;b.appendChild(d);});document.querySelectorAll(".del-cl").forEach(b=>{b.onclick=()=>{S.managed.client_rules.splice(parseInt(b.dataset.idx),1);renderClients();autoSave();};});}
function addClient(){const m=$("clMatcher").value.trim();if(!m){alert("客户端标识不能为空");return;}S.managed.client_rules.push({matcher:m,group_name:$("clGroup").value.trim()});$("clMatcher").value="";$("clGroup").value="";renderClients();autoSave();}

// -- IP Rules --
function renderIpRules(){const b=$("ipRuleList");b.innerHTML="";const it=S.managed.ip_rules||[];if(!it.length){b.innerHTML=`<div class="card empty">还没有 IP 规则。</div>`;return;}const lb={"ignore-ip":"忽略IP","whitelist-ip":"白名单","blacklist-ip":"黑名单","bogus-nxdomain":"假NX","ip-alias":"IP别名","ip-rules":"IP规则"};it.forEach((x,i)=>{const d=document.createElement("div");d.className="card rc";d.innerHTML=`<div class="rm"><span class="bd">${esc(lb[x.rule_type]||x.rule_type)}</span> <strong>${esc(x.target)}</strong>${x.options?` <code>${esc(x.options)}</code>`:''}</div><button class="ghost del-ir" data-idx="${i}">删除</button>`;b.appendChild(d);});document.querySelectorAll(".del-ir").forEach(b=>{b.onclick=()=>{S.managed.ip_rules.splice(parseInt(b.dataset.idx),1);renderIpRules();autoSave();};});}
function addIpRule(){const t=$("irTarget").value.trim();if(!t){alert("目标不能为空");return;}S.managed.ip_rules.push({rule_type:$("irType").value,target:t,options:$("irOptions").value.trim(),group_name:""});$("irTarget").value="";$("irOptions").value="";renderIpRules();autoSave();}

// -- IP Sets --

// -- Conf Files --
function renderConfFiles(){
  const b=$("confFileList");b.innerHTML="";
  const it=S.includeFiles||[];
  if(!it.length){b.innerHTML=`<div class="card empty">当前没有任何 conf-file 引入。</div>`;return;}
  it.forEach((x,i)=>{
    const d=document.createElement("div");d.className="card rc";
    d.innerHTML=`<div class="rm"><span class="bd">conf-file</span> <code>${esc(x.path)}</code>${x.group?` <span class="dim">→ ${esc(x.group)}</span>`:''}${x.managed?` <span class="bd">WebUI托管</span>`:''}</div>${x.managed?`<button class="ghost" disabled>托管</button>`:`<button class="ghost del-cf" data-idx="${i}">删除</button>`}`;
    b.appendChild(d);
  });
  document.querySelectorAll(".del-cf").forEach(btn=>{
    btn.onclick=()=>{
      const idx=parseInt(btn.dataset.idx);
      const item=(S.includeFiles||[])[idx];
      if(!item||item.managed)return;
      S.confFiles=S.confFiles.filter(x=>!(x.path===item.path&&x.group===item.group));
      S.includeFiles=(S.includeFiles||[]).filter(x=>x.managed||!(x.path===item.path&&x.group===item.group));
      renderConfFiles();autoSave();
    };
  });
}
function addConfFile(){
  const p=$("cfPath").value.trim();
  const g=$("cfGroup").value.trim();
  if(!p){alert("路径不能为空");return;}
  if(S.confFiles.some(x=>x.path===p&&x.group===g)){alert("该 conf-file 已存在");return;}
  const item={path:p,group:g,managed:false};
  S.confFiles.push({path:p,group:g});
  S.includeFiles=[...(S.includeFiles||[]),item];
  $("cfPath").value="";$("cfGroup").value="";
  renderConfFiles();autoSave();
}

// ── Save ──────────────────────────────────────────────────────────────
function collectExtras(){
  const e={};
  function kv(k,v){if(v&&v.trim())e[k]=v.trim();}
  function kp(k,v){if(v&&v.trim())e[k]=`[::]:${v.trim()}`;}
  function cb(k,c){if(c)e[k]="yes";}
  kp("bind-tcp",$("s_bind_tcp").value);kv("user",$("s_user").value);kv("socket-buff-size",$("s_socket_buff").value);
  cb("no-daemon",$("s_no_daemon").checked);cb("restart-on-crash",$("s_restart_on_crash").checked);cb("expand-ptr-from-address",$("s_expand_ptr").checked);
  kv("rr-ttl-min",$("p_rr_ttl_min").value);kv("rr-ttl-max",$("p_rr_ttl_max").value);kv("rr-ttl-reply-max",$("p_rr_ttl_reply_max").value);
  kv("local-ttl",$("p_local_ttl").value);kv("max-reply-ip-num",$("p_max_reply_ip").value);kv("max-query-limit",$("p_max_query_limit").value);
  kv("tcp-idle-time",$("p_tcp_idle_time").value);kv("serve-expired-reply-ttl",$("p_serve_expired_reply_ttl").value);
  kv("serve-expired-prefetch-time",$("p_serve_expired_prefetch").value);kv("dualstack-ip-selection-threshold",$("p_dualstack_threshold").value);
  kv("cache-persist",$("p_cache_persist").value);kv("cache-file",$("p_cache_file").value);
  // Combine proxy name + address
  const pn=$("x_proxy_name").value.trim(), pa=$("x_proxy_addr").value.trim();
  if(pa)kv("proxy-server",pn?`${pa} -name ${pn}`:pa);
  kv("hosts-file",$("x_hosts_file").value);kv("ca-file",$("x_ca_file").value);kv("ca-path",$("x_ca_path").value);
  kv("log-file",$("l_log_file").value);kv("log-size",$("l_log_size").value);kv("log-num",$("l_log_num").value);
  cb("log-console",$("l_log_console").checked);cb("log-syslog",$("l_log_syslog").checked);
  kv("audit-file",$("l_audit_file").value);kv("audit-size",$("l_audit_size").value);kv("audit-num",$("l_audit_num").value);
  cb("audit-console",$("l_audit_console").checked);
  kv("force-qtype-SOA",$("a_force_qtype_soa").value);
  cb("force-AAAA-SOA",$("a_force_aaaa_soa").checked);cb("force-no-CNAME",$("a_force_no_cname").checked);
  kv("edns-client-subnet",$("a_ecs").value);kv("dns64",$("a_dns64").value);
  kv("ddns-domain",$("a_ddns_domain").value);kv("local-domain",$("a_local_domain").value);kv("dnsmasq-lease-file",$("a_dnsmasq_lease").value);
  // conf-file extras
  S.confFiles.forEach(cf=>e[`conf-file`]=(e[`conf-file`]||"")+(e[`conf-file`]?", ":"")+cf.path+(cf.group?` -group ${cf.group}`:""));
  return e;
}

function collectBasic(){
  return {
    server_name:$("s_server_name").value.trim(),
    binds:$("s_bind").value.split("\n").map(x=>x.trim()).filter(Boolean).map(p=>`[::]:${p}`),
    cache_size:$("p_cache_size").value.trim(),
    prefetch_domain:$("p_prefetch_domain").checked,
    serve_expired:$("p_serve_expired").checked,
    serve_expired_ttl:$("p_serve_expired_ttl").value.trim(),
    response_mode:$("p_response_mode").value,
    speed_check_mode:$("p_speed_check_mode").value.trim(),
    dualstack_ip_selection:$("p_dualstack").checked,
    log_level:$("l_log_level").value,
    mdns_lookup:$("s_mdns_lookup").checked,
    audit_enable:$("l_audit_enable").checked,
    upstreams:S.upstreams,
    extras:collectExtras(),
  };
}

function buildManagedText(){
  let lines=["# ═══════════════════════════════════════════","#  域名集合与分流规则","#  由 SmartDNS Plus UI 生成","# ═══════════════════════════════════════════",""];
  const m=S.managed;
  const allDS=[...(m.domain_sets||[])];
  const seenDS=new Set(allDS.map(x=>x.name));
  (S.config&&S.config.rule_set_assets||[]).forEach(a=>{if(!seenDS.has(a.name)){seenDS.add(a.name);allDS.push({name:a.name,file:a.file||"",set_type:a.set_type||"list"});}});
  if(allDS.length){lines.push("# ── 域名集合 ──");allDS.forEach(ds=>lines.push(`domain-set -name ${ds.name} -type ${ds.set_type||"list"} -file ${ds.file}`));lines.push("");}
  if(m.ip_sets&&m.ip_sets.length){lines.push("# ── IP 集合 ──");m.ip_sets.forEach(is=>lines.push(`ip-set -name ${is.name} -file ${is.file}`));lines.push("");}
  const allRT=[...(m.routing_items||[])];
  const deleted=deletedRoutingKeys();
  const seenRT=new Set(allRT.map(x=>x.rule_type+"|"+x.target+"|"+x.value));
  (S.config&&S.config.routing_items||[]).forEach(r=>{if(r.rule_type!=="domain-set"){const k=r.rule_type+"|"+r.target+"|"+(r.value||"");if(!seenRT.has(k)&&!deleted.has(k)){seenRT.add(k);allRT.push({rule_type:r.rule_type,target:r.target,value:r.value||""});}}});
  // Split: nameserver (分流) vs others (规则)
  const ns=allRT.filter(r=>r.rule_type==="nameserver");
  const others=allRT.filter(r=>r.rule_type!=="nameserver");
  if(ns.length){
    lines.push("# ── 分流 ──");
    ns.forEach(r=>{
      const target=(r.target||"").trim();
      if(!target)return;
      if(target.startsWith("/")){
        lines.push(r.value?`${r.rule_type} ${target}/${r.value}`:`${r.rule_type} ${target}`);
      }else{
        lines.push(r.value?`${r.rule_type} ${target}/${r.value}`:`${r.rule_type} ${target}`);
      }
    });
    lines.push("");
  }
  if(others.length){lines.push("# ── 规则 ──");others.forEach(r=>{let tgt=r.target;if(!tgt.endsWith("/"))tgt+="/";lines.push(r.value?`${r.rule_type} ${tgt} ${r.value}`:`${r.rule_type} ${tgt}`);});lines.push("");}
  if(m.rule_groups&&m.rule_groups.length){lines.push("# ── 规则组 ──");m.rule_groups.forEach(g=>{lines.push(g.inherit?`group-begin ${g.name} -inherit ${g.inherit}`:`group-begin ${g.name}`);(g.matchers||[]).forEach(mt=>lines.push(`group-match ${mt}`));lines.push("group-end");lines.push("");});}
  if(m.client_rules&&m.client_rules.length){lines.push("# ── 客户端规则 ──");m.client_rules.forEach(c=>lines.push(`client-rules ${c.matcher}`));lines.push("");}
  if(m.ip_rules&&m.ip_rules.length){lines.push("# ── IP 规则 ──");m.ip_rules.forEach(r=>lines.push(r.options?`${r.rule_type} ${r.target} ${r.options}`:`${r.rule_type} ${r.target}`));lines.push("");}
  if(lines.length<=4)lines.push("# 还没有规则配置。");
  lines.push("");return lines.join("\n");
}

let _saveTimer=null;
function autoSave(){
  clearTimeout(_saveTimer);
  _saveTimer=setTimeout(async()=>{
    const p={beginner:collectBasic(),rules_text:buildManagedText()};
    await api("/api/plus/save",{method:"POST",body:JSON.stringify(p)});
  },200);
}

function sleep(ms){
  return new Promise(resolve=>setTimeout(resolve,ms));
}

async function waitForServiceRecovery(){
  let lastError=null;
  for(let i=0;i<12;i++){
    try{
      await loadAll();
      return true;
    }catch(e){
      lastError=e;
      await sleep(1000);
    }
  }
  if(lastError)console.warn("服务重启后自动恢复失败:",lastError);
  return false;
}

async function applyAndRestart(){
  const p={beginner:collectBasic(),rules_text:buildManagedText()};
  const r=await api("/api/plus/save",{method:"POST",body:JSON.stringify(p)});
  if(r.validation&&!r.validation.ok){alert("保存失败:\n"+r.validation.errors.join("\n"));return;}
  await api("/api/plus/apply",{method:"POST"});
  $("statusLabel").textContent="配置已应用，等待 SmartDNS 重启...";
  alert("配置已应用，SmartDNS 正在重启。");
  const recovered=await waitForServiceRecovery();
  if(!recovered){
    $("statusLabel").textContent="配置已应用，SmartDNS 重启中，请稍后刷新。";
  }
}

// ── Preview ──────────────────────────────────────────────────────────
async function renderPreview(){
  const d=await api("/api/plus/config");
  $("previewText").value=d.working_config_text||d.main_config_text||"";
}

// ── Events ────────────────────────────────────────────────────────────
$("loginBtn").onclick=()=>login().catch(e=>alert("登录失败: "+e.message));
$("logoutBtn").onclick=()=>logout();
$("password").addEventListener("keydown",e=>{if(e.key==="Enter")login().catch(err=>alert("登录失败: "+err.message));});
$("reloadBtn").onclick=()=>loadAll().catch(e=>alert("刷新失败: "+e.message));
$("saveRestartBtn").onclick=()=>applyAndRestart().catch(e=>alert("重启失败: "+e.message));
$("addUpstreamModalBtn").onclick=()=>{$("upModal").classList.remove("hidden");refreshGroupSelects();};
$("upModalClose").onclick=()=>$("upModal").classList.add("hidden");
$("upModalCancel").onclick=()=>$("upModal").classList.add("hidden");
$("upModal").querySelector(".modal-bg").onclick=()=>$("upModal").classList.add("hidden");
$("addUpstreamBtn").onclick=()=>{addUpstream();$("upModal").classList.add("hidden");};
$("addSetBtn").onclick=()=>addSet();
$("addRoutingBtn").onclick=()=>addRouting();
$("addDomainRuleBtn").onclick=()=>addDomainRule();
$("addAddrCnameBtn").onclick=()=>addAddrCname();
$("addIpRuleBtn").onclick=()=>addIpRule();
$("addConfFileBtn").onclick=()=>addConfFile();

// ── Init ───────────────────────────────────────────────────────────────
window.onerror=function(msg,url,line){alert("JS错误 行"+line+": "+msg);return false;};
goto("service");checkAuth();
