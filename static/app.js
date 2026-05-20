let board,state,recommendation,history=[],selectedBear=false,legalBearMoves=[];
const svg=document.getElementById('board'); const modeEl=document.getElementById('mode');
const j=async(u,o)=>{const r=await fetch(u,o);const d=await r.json();if(!r.ok)throw new Error(d.error||'request failed');return d;};
const post=(u,b)=>j(u,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(b)});
const scale=n=>({x:(n.x+5)*70+40,y:(n.y+4)*70+40});
const ms=m=>m?`${m.type==='Hunter'?`Hunter#${m.hunterIndex}`:'Bear'}: ${m.from} -> ${m.to}`:'(none)';

async function refresh(){
  if(state.sideToMove==='Hunters') recommendation=await post('/api/recommend-hunter-move',state);
  else recommendation=null;
  legalBearMoves = state.sideToMove==='Bear' ? (await post('/api/legal-moves',state)).legalMoves.filter(m=>m.type==='Bear') : [];
  render();
}
function pushHistory(){ history.push(JSON.parse(JSON.stringify(state))); }
async function applyBest(){ pushHistory(); const out=await post('/api/apply-best-hunter-move',state); state=out.state; selectedBear=false; await refresh(); }
async function manualBear(to){ pushHistory(); const out=await post('/api/manual-bear-move',{state,to}); state=out.state; recommendation=out.recommendation; selectedBear=false; await refresh(); }

function render(){
  const mode=modeEl.value; const map=Object.fromEntries(board.nodes.map(n=>[n.id,scale(n)])); svg.innerHTML='';
  board.edges.forEach(([a,b])=>{const l=document.createElementNS('http://www.w3.org/2000/svg','line'); l.setAttribute('x1',map[a].x);l.setAttribute('y1',map[a].y);l.setAttribute('x2',map[b].x);l.setAttribute('y2',map[b].y);l.setAttribute('class','edge');svg.appendChild(l);});
  board.nodes.forEach(n=>{const p=map[n.id];const c=document.createElementNS('http://www.w3.org/2000/svg','circle');c.setAttribute('cx',p.x);c.setAttribute('cy',p.y);c.setAttribute('r',14);c.setAttribute('class','node');svg.appendChild(c);
  const t=document.createElementNS('http://www.w3.org/2000/svg','text');t.setAttribute('x',p.x+16);t.setAttribute('y',p.y+4);t.textContent=n.id;t.setAttribute('font-size','10');svg.appendChild(t);});
  const bear=map[state.bear]; const bc=document.createElementNS('http://www.w3.org/2000/svg','circle'); bc.setAttribute('cx',bear.x);bc.setAttribute('cy',bear.y);bc.setAttribute('r',11);bc.setAttribute('class','bear'); bc.onclick=()=>{if(state.sideToMove==='Bear') {selectedBear=true; render();}}; svg.appendChild(bc);
  state.hunters.forEach(h=>{const p=map[h];const c=document.createElementNS('http://www.w3.org/2000/svg','circle');c.setAttribute('cx',p.x);c.setAttribute('cy',p.y);c.setAttribute('r',11);c.setAttribute('class','hunter');svg.appendChild(c);});
  if(state.sideToMove==='Hunters' && recommendation?.recommendedMove){const m=recommendation.recommendedMove; const a=map[m.from], b=map[m.to]; const hl=document.createElementNS('http://www.w3.org/2000/svg','line'); hl.setAttribute('x1',a.x);hl.setAttribute('y1',a.y);hl.setAttribute('x2',b.x);hl.setAttribute('y2',b.y);hl.setAttribute('class','best'); svg.appendChild(hl);}
  if(state.sideToMove==='Bear' && selectedBear){ legalBearMoves.forEach(m=>{const p=map[m.to];const d=document.createElementNS('http://www.w3.org/2000/svg','circle');d.setAttribute('cx',p.x);d.setAttribute('cy',p.y);d.setAttribute('r',7);d.setAttribute('class','dest'); d.onclick=()=>manualBear(m.to); svg.appendChild(d);}); }

  const phase = state.sideToMove==='Hunters' ? "Hunters' turn: apply the recommended hunter move." : "Bear's turn: manually move the bear to match the real board.";
  document.getElementById('status').textContent=`${phase} Hunter turns used: ${state.hunterTurnsUsed}/${board.turnLimit}`;
  document.getElementById('details').textContent = JSON.stringify({mode, state, recommendation, assistantNote:"The bear may choose any legal response. This hunter move remains winning against all legal bear responses, assuming the state is still within the solved winning region."}, null, 2);
}

window.onload=async()=>{
  board=await j('/api/board'); state=board.start;
  document.getElementById('reset').onclick=async()=>{state=board.start;history=[];selectedBear=false;await refresh();};
  document.getElementById('applyBest').onclick=async()=>{if(state.sideToMove!=='Hunters') return alert('Move the bear manually first.'); await applyBest();};
  document.getElementById('undo').onclick=async()=>{const prev=history.pop(); if(prev){state=prev;selectedBear=false;await refresh();}};
  document.getElementById('recalc').onclick=refresh; modeEl.onchange=render;
  await refresh();
};
