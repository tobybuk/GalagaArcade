// Automated player for the unmodified production module. Read-only inspection
// chooses ordinary left/right/fire keys; it never changes RAM or game rules.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const module=await WebAssembly.compile(readFileSync(new URL('../dist/galaga.wasm',import.meta.url)));
const {exports:api}=await WebAssembly.instantiate(module);
const game=api.game_new();
const destinationStage=Number(process.argv[2]||4);
const captureMode=process.argv.includes('--capture');
const initialsMode=process.argv.includes('--initials');
const status=field=>api.game_status(game,field);
function step(input=0) {
  api.game_step(game,input);assert.equal(status(4),0,`Original routine at frame ${status(0)}`);
}
function press(input) {step(input);step(input);step();step();}
assert.equal(status(20),1,'Production module must begin with the original cold-start test route');
for (let frame=0;frame<400 && status(20);frame++) step();
assert.equal(status(20),0,'Cold-start test route must reach attract mode');
assert.equal(status(10),1);
step();step(); // original 51xx idle samples after credit-mode handoff
press(8);press(16);
let previous=Array.from({length:64},()=>({x:0,y:0,state:128}));
const stages=[];let target=122;let maximumScore=0;let captured=false;let joined=false;let capturedIndex=-1;
let primeShotFrame=-1000;
const history=[];
let initialsEntered=false;let initialsRank=0;
let beamSeen=false,pullSeen=false,transferSeen=false,rescueSeen=false;
for (let frame=0;frame<60000 && status(10)===3;frame++) {
  if (initialsMode && !initialsEntered && status(32)) {
    initialsRank=status(0x311);
    for (const letter of [27,30,28]) { // RUS, entered through original controls
      let moved=0;
      while (status(32) && status(0x500+status(0x310))!==letter && moved<600) {step(2);moved++;}
      assert.equal(status(0x500+status(0x310)),letter);
      step();step();press(4);
    }
    initialsEntered=true;console.log(JSON.stringify({frame,initialsEntered,initialsRank}));continue;
  }
  const objects=previous.map((_,i)=>{
    const packed=status(0x100+i);
    return {x:packed&255,y:packed>>>8&511,state:packed>>>17&255,code:packed>>>25};
  });
  const fighter=objects[49];
  const threats=[];const targets=[];
  for (let i=0;i<60;i++) {
    const o=objects[i];if (!o.x || o.state>=128 || o.state===4 || o.state===5) continue;
    const vx=o.x-previous[i].x,vy=o.y-previous[i].y;
    if (i<48 && o.state!==0) {
      const time=Math.max(0,(fighter.y-o.y)/6);
      const aim=o.state===1 ? Math.max(18,Math.min(224,o.x+vx*time)) : o.x;
      targets.push({id:i,aim,cost:Math.abs(aim-fighter.x)+Math.max(0,fighter.y-o.y)*0.04});
    }
    if ((i<48 || i>=52) && o.y>130 && o.y<330 && Math.abs(vx)<12 && Math.abs(vy)<12) {
      threats.push({x:o.x,y:o.y,vx,vy});
    }
  }
  targets.sort((a,b)=>a.cost-b.cost);
  if (targets.length) target=targets[0].aim;
  let firing=true;let priming=false;
  if (captureMode && !captured) {
    firing=false;
    const boss=objects[status(22)>>1];
    target=(status(23) && boss?.x) ? boss.x : 122;
  } else if (captureMode && !joined) {
    const boss=objects[24+capturedIndex];
    const green=(status(0x200+24+capturedIndex)&255)===0;
    if (boss?.x && boss.state!==1) {
      const time=Math.max(0,(fighter.y-boss.y)/6)+3;
      const velocity=boss.x-previous[24+capturedIndex].x;
      target=Math.max(18,Math.min(224,boss.x+velocity*time));
    }
    else if (green) {
      const old=history[0]?.[24+capturedIndex]||boss;
      const velocity=Math.max(-1,Math.min(1,(boss.x-old.x)/Math.max(1,history.length)));
      target=Math.max(18,Math.min(224,boss.x+velocity*(Math.max(0,(fighter.y-boss.y)/6)+3)));priming=true;
    } else {
      const other=targets.find(t=>t.id>=4 && t.id!==24+capturedIndex);
      if (other) target=other.aim;
    }
    firing=priming ? frame-primeShotFrame>90 && Math.abs(target-fighter.x)<5 :
      !(boss?.state===1 && Math.abs(boss.x-fighter.x)<30);
    const ship=objects[capturedIndex];
    if (ship?.x && ship.state!==1 && ship.y>=boss.y) {
      const time=Math.max(0,(fighter.y-ship.y)/6)+3;
      const velocity=ship.x-previous[capturedIndex].x;
      if (Math.abs(ship.x+velocity*time-fighter.x)<20) firing=false;
    }
  }
  let best=0,bestCost=Infinity;
  for (const direction of [0,-1,1]) {
    let cost=Math.abs(Math.max(17,Math.min(225,fighter.x+direction*(priming?7.5:18)))-target);
    if (captureMode && !captured && status(24)) cost*=12;
    for (const threat of threats) {
      for (let t=3;t<=36;t+=3) {
        const x=Math.max(17,Math.min(225,fighter.x+direction*1.5*Math.max(0,t-1)));
        const dy=Math.abs(threat.y+threat.vy*t-fighter.y);
        if (dy<20) cost+=Math.max(0,23-Math.abs(threat.x+threat.vx*t-x))*(20-dy)*0.6;
      }
    }
    if (cost<bestCost) {bestCost=cost;best=direction;}
  }
  const shotsBefore=status(5);
  if (initialsMode && status(18)===255) firing=false;
  step((best<0?1:best>0?2:0)|(firing && frame%8<2?4:0));
  if (priming && status(5)!==shotsBefore) primeShotFrame=frame;
  previous=objects;maximumScore=Math.max(maximumScore,status(7));
  history.push(objects);if (history.length>16) history.shift();
  const stage=status(9);if (stage && !stages.includes(stage)) {stages.push(stage);console.log(JSON.stringify({frame,stage,score:status(7),reserves:status(18)}));}
  const capturedObjects=objects.slice(0,4).map((o,i)=>o.x!==0 && o.state===1 && o.code<=7 && (status(0x200+i)&255)===7);
  const nowCaptured=capturedObjects.some(Boolean);
  if (nowCaptured && !captured) {
    capturedIndex=capturedObjects.findIndex(Boolean);
    console.log(JSON.stringify({frame,captured:true,capturedIndex}));
  }
  captured ||= nowCaptured;
  joined ||= status(21)!==0;
  beamSeen ||= status(24)!==0;pullSeen ||= status(25)!==0;
  transferSeen ||= status(26)!==0;rescueSeen ||= status(27)!==0;
  if (captureMode && frame%2000===0) console.log(JSON.stringify({frame,fighter,shots:status(5),fire:status(16),
    boss:status(22),beam:status(24),pull:status(25),captureTask:status(26),rescueTask:status(27),
    reserves:status(18),restart:status(28),controls:status(29),flying:status(30),enemies:status(31),
    capturedShip:capturedIndex>=0?objects[capturedIndex]:null,
    carrier:capturedIndex>=0?objects[24+capturedIndex]:null,
    carrierColour:capturedIndex>=0?(status(0x200+24+capturedIndex)&255):null}));
  if (captureMode ? joined : !initialsMode && stage>=destinationStage) break;
}
const initials=initialsRank ? Array.from({length:3},(_,i)=>status(0x33e+(initialsRank-1)*3+i)) : [];
const report={stages,maximumScore,hits:status(8),frames:status(0),state:status(10),captured,joined,
  beamSeen,pullSeen,transferSeen,rescueSeen,initialsEntered,initials};
console.log(JSON.stringify(report));api.game_free(game);
if (initialsMode) {assert.ok(initialsEntered);assert.deepEqual(initials,[27,30,28]);assert.equal(report.state,1);}
else if (captureMode) {
  assert.ok(joined && beamSeen && pullSeen && transferSeen && rescueSeen,
    'Ordinary inputs must execute the original beam, pull, transfer, rescue and joined-fighter sequence');
}
else assert.ok(stages.includes(destinationStage),`Ordinary player inputs should reach stage ${destinationStage}`);
