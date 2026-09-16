// This is a build-time Brainfuck macro assembler, never a runtime evaluator.
// All emitted algorithms use only the eight Brainfuck instructions.
import {writeFileSync} from 'node:fs';
const SCALE=10n**24n;
const C={pi:3141592653589793238462643n,e:2718281828459045235360287n,ln2:693147180559945309417232n,ln10:2302585092994045684017991n};
export const MUL='[>[->+>+<<]>>[-<<+>>]<<<-]';
export const DIV='[->-[>+>>]>[[-<+>]+>+>>]<<<<<]';
class BF {
 constructor(){this.code='';this.p=0;this.next=8;this.high=8;}
 at(n){this.code+=(n>this.p?'>':'<').repeat(Math.abs(n-this.p));this.p=n;}
 emit(s){this.code+=s;}
 alloc(n=1){const x=this.next;this.next+=n;this.high=Math.max(this.high,this.next);return x;}
 pair(){return {v:this.alloc(),s:this.alloc()};}
 scope(fn){const n=this.next;fn();this.next=n;}
 clear(a){this.at(a);this.emit('[-]');}
 inc(a,n=1){this.at(a);this.emit((n>=0?'+':'-').repeat(Math.abs(n)));}
 loop(a,fn){this.at(a);this.emit('[');fn();this.at(a);this.emit(']');}
 move(a,b,factor=1){this.loop(a,()=>{this.inc(a,-1);this.inc(b,factor)});}
 set(a,value){value=BigInt(value);this.scope(()=>{const t=this.alloc();this.clear(a);this.clear(t);for(const digit of value.toString()){this.move(a,t,10);this.move(t,a);this.inc(a,Number(digit));}});}
 copy(a,b){if(a===b)return;this.scope(()=>{const t=this.alloc();this.clear(t);this.clear(b);this.loop(a,()=>{this.inc(a,-1);this.inc(b);this.inc(t)});this.move(t,a);});}
 addU(a,b,out){this.scope(()=>{const x=this.alloc(),y=this.alloc();this.copy(a,x);this.copy(b,y);this.clear(out);this.move(x,out);this.move(y,out);});}
 subU(a,b,out){this.scope(()=>{const x=this.alloc(),y=this.alloc();this.copy(a,x);this.copy(b,y);this.move(y,x,-1);this.clear(out);this.move(x,out);});}
 branch(a,yes,no=()=>{}){this.scope(()=>{const f=this.alloc(),e=this.alloc();this.copy(a,f);this.set(e,1);this.loop(f,()=>{this.clear(f);this.clear(e);yes()});this.loop(e,()=>{this.clear(e);no()});});}
 bool(a,out){this.branch(a,()=>this.set(out,1),()=>this.clear(out));}
 mulU(a,b,out){this.scope(()=>{const x=this.alloc(4);for(let i=0;i<4;i++)this.clear(x+i);this.copy(a,x);this.copy(b,x+1);this.at(x);this.emit(MUL);this.copy(x+2,out);});}
 divU(a,b,q,r){this.scope(()=>{const x=this.alloc(6);for(let i=0;i<6;i++)this.clear(x+i);this.copy(a,x);this.copy(b,x+1);this.inc(x+2);this.at(x);this.emit(DIV);this.copy(x+3,q);if(r!==undefined){this.inc(x+2,-1);this.copy(x+2,r);}});}
 ge(a,b,out){this.scope(()=>{const q=this.alloc();this.branch(b,()=>{this.divU(a,b,q);this.bool(q,out)},()=>this.set(out,1));});}
 eq(a,b,out){this.scope(()=>{const p=this.alloc(),q=this.alloc();this.ge(a,b,p);this.ge(b,a,q);this.branch(p,()=>this.copy(q,out),()=>this.clear(out));});}
 constant(out,n){n=BigInt(n);this.set(out.v,n<0?-n:n);this.set(out.s,n<0?1:0);}
 clone(a,out){this.copy(a.v,out.v);this.copy(a.s,out.s);}
 clean(a){this.branch(a.v,()=>{},()=>this.clear(a.s));}
 neg(a,out){this.clone(a,out);this.branch(out.s,()=>this.clear(out.s),()=>this.set(out.s,1));this.clean(out);}
 add(a,b,out,minus=false){this.scope(()=>{const x=this.pair(),y=this.pair(),same=this.alloc(),greater=this.alloc();this.clone(a,x);this.clone(b,y);if(minus)this.neg(y,y);this.eq(x.s,y.s,same);this.branch(same,()=>{this.addU(x.v,y.v,out.v);this.copy(x.s,out.s)},()=>{this.ge(x.v,y.v,greater);this.branch(greater,()=>{this.subU(x.v,y.v,out.v);this.copy(x.s,out.s)},()=>{this.subU(y.v,x.v,out.v);this.copy(y.s,out.s)})});this.clean(out);});}
 mul(a,b,out){this.scope(()=>{const v=this.alloc(),scale=this.alloc(),sign=this.alloc();this.eq(a.s,b.s,sign);this.mulU(a.v,b.v,v);this.set(scale,SCALE);this.divU(v,scale,out.v);this.branch(sign,()=>this.clear(out.s),()=>this.set(out.s,1));this.clean(out);});}
 div(a,b,out){this.scope(()=>{const v=this.alloc(),scale=this.alloc(),sign=this.alloc();this.branch(b.v,()=>{this.eq(a.s,b.s,sign);this.set(scale,SCALE);this.mulU(a.v,scale,v);this.divU(v,b.v,out.v);this.branch(sign,()=>this.clear(out.s),()=>this.set(out.s,1));this.clean(out);},()=>{this.set(5,1);this.constant(out,0)});});}
 sqrt(a,out){this.scope(()=>{const n=this.alloc(),guess=this.alloc(),next=this.alloc(),q=this.alloc(),two=this.alloc(),condition=this.alloc(),scale=this.alloc();this.branch(a.s,()=>{this.set(5,2);this.constant(out,0)},()=>{this.set(scale,SCALE);this.mulU(a.v,scale,n);this.copy(n,guess);this.set(two,2);this.divU(n,two,next);this.inc(next);this.ge(guess,next,condition);this.loop(condition,()=>{this.copy(next,guess);this.divU(n,guess,q);this.addU(guess,q,q);this.divU(q,two,next);this.ge(next,guess,condition);this.branch(condition,()=>this.clear(condition),()=>this.set(condition,1));});this.copy(guess,out.v);this.clear(out.s);});});}
 ln(a,out){this.scope(()=>{const x=this.pair(),k=this.pair(),one=this.pair(),two=this.pair(),z=this.pair(),z2=this.pair(),term=this.pair(),sum=this.pair(),den=this.pair(),piece=this.pair(),l2=this.pair(),cond=this.alloc(),count=this.alloc();this.clone(a,x);this.constant(k,0);this.constant(one,SCALE);this.constant(two,2n*SCALE);this.branch(x.s,()=>this.set(5,2));this.branch(x.v,()=>{
 this.ge(x.v,two.v,cond);this.loop(cond,()=>{this.div(x,two,x);this.add(k,one,k);this.ge(x.v,two.v,cond)});
 this.ge(x.v,one.v,cond);this.branch(cond,()=>this.clear(cond),()=>this.set(cond,1));this.loop(cond,()=>{this.mul(x,two,x);this.add(k,one,k,true);this.ge(x.v,one.v,cond);this.branch(cond,()=>this.clear(cond),()=>this.set(cond,1));});
 this.add(x,one,z,true);this.add(x,one,piece);this.div(z,piece,z);this.mul(z,z,z2);this.clone(z,term);this.clone(z,sum);this.constant(den,SCALE);this.set(count,45);
 this.loop(count,()=>{this.mul(term,z2,term);this.add(den,two,den);this.div(term,den,piece);this.add(sum,piece,sum);this.inc(count,-1)});
 this.mul(sum,two,sum);this.constant(l2,C.ln2);this.mul(k,l2,piece);this.add(sum,piece,out);
 },()=>{this.set(5,2);this.constant(out,0)});});}
 exp(a,out){this.scope(()=>{const x=this.pair(),one=this.pair(),two=this.pair(),half=this.pair(),term=this.pair(),sum=this.pair(),den=this.pair(),limit=this.alloc(),bad=this.alloc(),cond=this.alloc(),squares=this.alloc(),count=this.alloc();this.clone(a,x);this.clear(x.s);this.set(limit,710n*SCALE);this.ge(x.v,limit,bad);this.branch(bad,()=>{this.set(5,3);this.constant(out,0)},()=>{
 this.constant(one,SCALE);this.constant(two,2n*SCALE);this.constant(half,SCALE/2n);this.clear(squares);this.ge(x.v,half.v,cond);this.loop(cond,()=>{this.div(x,two,x);this.inc(squares);this.ge(x.v,half.v,cond)});
 this.clone(one,term);this.clone(one,sum);this.constant(den,0);this.set(count,35);
 this.loop(count,()=>{this.add(den,one,den);this.mul(term,x,term);this.div(term,den,term);this.add(sum,term,sum);this.inc(count,-1)});
 this.loop(squares,()=>{this.mul(sum,sum,sum);this.inc(squares,-1)});
 this.branch(a.s,()=>this.div(one,sum,out),()=>this.clone(sum,out));
 });});}
 trig(a,out,cosine=false){this.scope(()=>{const x=this.pair(),pi=this.pair(),full=this.alloc(),q=this.alloc(),r=this.alloc(),deg=this.pair(),x2=this.pair(),term=this.pair(),sum=this.pair(),den=this.pair(),n=this.alloc(),m=this.alloc(),scale=this.alloc(),count=this.alloc();this.clone(a,x);this.branch(4,()=>{},()=>{this.scope(()=>{const limit=this.alloc(),tooLarge=this.alloc();this.set(limit,1000000000000n*SCALE+1n);this.ge(x.v,limit,tooLarge);this.branch(tooLarge,()=>this.set(5,6));});});this.constant(pi,C.pi);this.branch(4,()=>{this.mul(x,pi,x);this.constant(deg,180n*SCALE);this.div(x,deg,x)});this.set(full,2n*C.pi);this.divU(x.v,full,q,r);this.copy(r,x.v);
 this.mul(x,x,x2);this.neg(x2,x2);if(cosine)this.constant(term,SCALE);else this.clone(x,term);this.clone(term,sum);this.set(n,cosine?0:1);this.set(scale,SCALE);this.set(count,42);
 this.loop(count,()=>{this.inc(n);this.copy(n,m);this.inc(n);this.mulU(m,n,den.v);this.mulU(den.v,scale,den.v);this.clear(den.s);this.mul(term,x2,term);this.div(term,den,term);this.add(sum,term,sum);this.inc(count,-1)});this.clone(sum,out);
 });}
 pow(a,b,out){this.scope(()=>{const scale=this.alloc(),n=this.alloc(),rem=this.alloc(),x=this.pair(),result=this.pair(),one=this.pair(),two=this.alloc(),bit=this.alloc(),log=this.pair(),product=this.pair();this.set(scale,SCALE);this.divU(b.v,scale,n,rem);this.constant(one,SCALE);
 this.branch(rem,()=>{this.branch(a.s,()=>{this.set(5,2);this.constant(out,0)},()=>{this.branch(a.v,()=>{this.ln(a,log);this.mul(log,b,product);this.exp(product,out)},()=>{this.branch(b.s,()=>this.set(5,1));this.constant(out,0)})});},()=>{
 this.clone(a,x);this.clone(one,result);this.set(two,2);
 this.loop(n,()=>{this.divU(n,two,n,bit);this.branch(bit,()=>this.mul(result,x,result));this.branch(n,()=>this.mul(x,x,x));});
 this.branch(b.s,()=>this.div(one,result,out),()=>this.clone(result,out));
 });});}
 factorial(a,out){this.scope(()=>{const scale=this.alloc(),n=this.alloc(),rem=this.alloc(),limit=this.alloc(),bad=this.alloc();this.set(scale,SCALE);this.divU(a.v,scale,n,rem);this.set(limit,171);this.ge(n,limit,bad);this.branch(a.s,()=>this.set(bad,1));this.branch(rem,()=>this.set(bad,1));this.branch(bad,()=>{this.set(5,4);this.constant(out,0)},()=>{this.constant(out,SCALE);this.loop(n,()=>{this.mulU(out.v,n,out.v);this.inc(n,-1)});});});}
}
const ops=['add','sub','mul','div','neg','abs','sqrt','ln','log','sin','cos','tan','pow','factorial','pi','e'];
const manifest={dialect:'arbitrary precision nonnegative integer cells; decrement below zero is an error',scale:SCALE.toString(),abi:{aMagnitude:0,aNegative:1,bMagnitude:2,bNegative:3,degrees:4,error:5,resultMagnitude:6,resultNegative:7},programs:{}};
for(const op of ops){const g=new BF(),a={v:0,s:1},b={v:2,s:3},out={v:6,s:7};
 if(op==='add'||op==='sub')g.add(a,b,out,op==='sub');else if(op==='abs'){g.copy(a.v,out.v);g.clear(out.s)}else if(op==='pi'||op==='e')g.constant(out,C[op]);else if(op==='log'){g.scope(()=>{const l=g.pair(),d=g.pair();g.ln(a,l);g.constant(d,C.ln10);g.div(l,d,out)})}else if(op==='cos')g.trig(a,out,true);else if(op==='sin')g.trig(a,out);else if(op==='tan'){g.scope(()=>{const s=g.pair(),c=g.pair(),tiny=g.alloc(),ok=g.alloc();g.trig(a,s);g.trig(a,c,true);g.set(tiny,10000);g.ge(c.v,tiny,ok);g.branch(ok,()=>g.div(s,c,out),()=>{g.set(5,5);g.constant(out,0)})})}else g[op](a,...(op==='mul'||op==='div'||op==='pow'?[b]:[]),out);
 g.at(6);writeFileSync(new URL(`${op}.bf`,import.meta.url),g.code+'\n');manifest.programs[op]={instructions:g.code.length,cells:g.high};
}
writeFileSync(new URL('manifest.json',import.meta.url),JSON.stringify(manifest,null,2)+'\n');
console.log('Generated 16 pure Brainfuck numeric programs');
