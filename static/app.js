let board,state,legal=[],evalData,selected=null;
const svg=document.getElementById('board');
async function j(u,o){const r=await fetch(u,o);const d=await r.json();if(!r.ok)throw new Error(d.error||'request failed');return d;}
function scale(n){return {x:(n.x+5)*70+40,y:(n.y+4)*70+40};}
function moveStr(m){return m.type==='Hunter'?`H${m.hunterIndex} ${m.from}->${m.to}`:`B ${m.from}->${m.to}`}
async function refresh(){legal=(await j('/api/legal-moves',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(state)})).legalMoves; evalData=await j('/api/evaluate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(state)}); render();}
function render(){const map=Object.fromEntries(board.nodes.map(n=>[n.id,scale(n)])); svg.innerHTML=''; board.edges.forEach(([a,b])=>{const l=document.createElementNS('http://www.w3.org/2000/svg','line'); l.setAttribute('x1',map[a].x);l.setAttribute('y1',map[a].y);l.setAttribute('x2',map[b].x);l.setAttribute('y2',map[b].y);l.setAttribute('class','edge');svg.appendChild(l)});
board.nodes.forEach(n=>{const p=map[n.id];const c=document.createElementNS('http://www.w3.org/2000/svg','circle');c.setAttribute('cx',p.x);c.setAttribute('cy',p.y);c.setAttribute('r',14);c.setAttribute('class','node');svg.appendChild(c);
const t=document.createElementNS('http://www.w3.org/2000/svg','text');t.setAttribute('x',p.x+16);t.setAttribute('y',p.y+4);t.textContent=n.id; t.setAttribute('font-size','10'); svg.appendChild(t);
});
function piece(id,cls,txt){const p=map[id];const c=document.createElementNS('http://www.w3.org/2000/svg','circle');c.setAttribute('cx',p.x);c.setAttribute('cy',p.y);c.setAttribute('r',11);c.setAttribute('class',cls);c.onclick=()=>selectPiece(id,txt);svg.appendChild(c)}
piece(state.bear,'bear','Bear'); state.hunters.forEach((h,i)=>piece(h,'hunter','Hunter'+i));
if(selected){ legal.filter(m=>(m.type==='Bear'&&state.sideToMove==='Bear'&&m.from===selected)||(m.type==='Hunter'&&state.sideToMove==='Hunters'&&m.from===selected)).forEach(m=>{const p=map[m.to];const c=document.createElementNS('http://www.w3.org/2000/svg','circle');c.setAttribute('cx',p.x);c.setAttribute('cy',p.y);c.setAttribute('r',7);c.setAttribute('class','dest');c.onclick=()=>playMove(m);svg.appendChild(c);}); }
document.getElementById('status').textContent=`Turn: ${state.sideToMove} | hunterTurnsUsed: ${state.hunterTurnsUsed}/${board.turnLimit}`;
document.getElementById('eval').textContent=JSON.stringify(evalData,null,2);
}
function selectPiece(id,label){ if((state.sideToMove==='Bear'&&label!=='Bear')||(state.sideToMove==='Hunters'&&!label.startsWith('Hunter')))return; selected=id; render(); }
async function playMove(m){const out=await j('/api/apply-move',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({state,mv:m})}); state=out.state; selected=null; await refresh();}
window.onload=async()=>{board=await j('/api/board'); state=board.start; document.getElementById('reset').onclick=async()=>{state=board.start;selected=null;await refresh()}; document.getElementById('refresh').onclick=refresh; document.getElementById('best').onclick=async()=>{if(evalData?.bestMove)await playMove(evalData.bestMove)}; await refresh();};
