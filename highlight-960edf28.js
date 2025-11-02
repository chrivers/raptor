/*!
  Highlight.js v11.11.1 (git: 5697ae5187)
  (c) 2006-2025 Josh Goebel <hello@joshgoebel.com> and other contributors
  License: BSD-3-Clause
 */
var hljs=function(){"use strict";function e(n){
return n instanceof Map?n.clear=n.delete=n.set=()=>{
throw Error("map is read-only")}:n instanceof Set&&(n.add=n.clear=n.delete=()=>{
throw Error("set is read-only")
}),Object.freeze(n),Object.getOwnPropertyNames(n).forEach((t=>{
const s=n[t],i=typeof s;"object"!==i&&"function"!==i||Object.isFrozen(s)||e(s)
})),n}class n{constructor(e){
void 0===e.data&&(e.data={}),this.data=e.data,this.isMatchIgnored=!1}
ignoreMatch(){this.isMatchIgnored=!0}}function t(e){
return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#x27;")
}function s(e,...n){const t=Object.create(null);for(const n in e)t[n]=e[n]
;return n.forEach((e=>{for(const n in e)t[n]=e[n]})),t}const i=e=>!!e.scope
;class a{constructor(e,n){
this.buffer="",this.classPrefix=n.classPrefix,e.walk(this)}addText(e){
this.buffer+=t(e)}openNode(e){if(!i(e))return;const n=((e,{prefix:n})=>{
if(e.startsWith("language:"))return e.replace("language:","language-")
;if(e.includes(".")){const t=e.split(".")
;return[`${n}${t.shift()}`,...t.map(((e,n)=>`${e}${"_".repeat(n+1)}`))].join(" ")
}return`${n}${e}`})(e.scope,{prefix:this.classPrefix});this.span(n)}
closeNode(e){i(e)&&(this.buffer+="</span>")}value(){return this.buffer}span(e){
this.buffer+=`<span class="${e}">`}}const r=(e={})=>{const n={children:[]}
;return Object.assign(n,e),n};class o{constructor(){
this.rootNode=r(),this.stack=[this.rootNode]}get top(){
return this.stack[this.stack.length-1]}get root(){return this.rootNode}add(e){
this.top.children.push(e)}openNode(e){const n=r({scope:e})
;this.add(n),this.stack.push(n)}closeNode(){
if(this.stack.length>1)return this.stack.pop()}closeAllNodes(){
for(;this.closeNode(););}toJSON(){return JSON.stringify(this.rootNode,null,4)}
walk(e){return this.constructor._walk(e,this.rootNode)}static _walk(e,n){
return"string"==typeof n?e.addText(n):n.children&&(e.openNode(n),
n.children.forEach((n=>this._walk(e,n))),e.closeNode(n)),e}static _collapse(e){
"string"!=typeof e&&e.children&&(e.children.every((e=>"string"==typeof e))?e.children=[e.children.join("")]:e.children.forEach((e=>{
o._collapse(e)})))}}class c extends o{constructor(e){super(),this.options=e}
addText(e){""!==e&&this.add(e)}startScope(e){this.openNode(e)}endScope(){
this.closeNode()}__addSublanguage(e,n){const t=e.root
;n&&(t.scope="language:"+n),this.add(t)}toHTML(){
return new a(this,this.options).value()}finalize(){
return this.closeAllNodes(),!0}}function l(e){
return e?"string"==typeof e?e:e.source:null}function d(e){return h("(?=",e,")")}
function g(e){return h("(?:",e,")*")}function u(e){return h("(?:",e,")?")}
function h(...e){return e.map((e=>l(e))).join("")}function b(...e){const n=(e=>{
const n=e[e.length-1]
;return"object"==typeof n&&n.constructor===Object?(e.splice(e.length-1,1),n):{}
})(e);return"("+(n.capture?"":"?:")+e.map((e=>l(e))).join("|")+")"}
function p(e){return RegExp(e.toString()+"|").exec("").length-1}
const m=/\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./
;function f(e,{joinWith:n}){let t=0;return e.map((e=>{t+=1;const n=t
;let s=l(e),i="";for(;s.length>0;){const e=m.exec(s);if(!e){i+=s;break}
i+=s.substring(0,e.index),
s=s.substring(e.index+e[0].length),"\\"===e[0][0]&&e[1]?i+="\\"+(Number(e[1])+n):(i+=e[0],
"("===e[0]&&t++)}return i})).map((e=>`(${e})`)).join(n)}
const _="[a-zA-Z]\\w*",E="[a-zA-Z_]\\w*",N="\\b\\d+(\\.\\d+)?",w="(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)",y="\\b(0b[01]+)",x={
begin:"\\\\[\\s\\S]",relevance:0},v={scope:"string",begin:"'",end:"'",
illegal:"\\n",contains:[x]},O={scope:"string",begin:'"',end:'"',illegal:"\\n",
contains:[x]},M=(e,n,t={})=>{const i=s({scope:"comment",begin:e,end:n,
contains:[]},t);i.contains.push({scope:"doctag",
begin:"[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
end:/(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,excludeBegin:!0,relevance:0})
;const a=b("I","a","is","so","us","to","at","if","in","it","on",/[A-Za-z]+['](d|ve|re|ll|t|s|n)/,/[A-Za-z]+[-][a-z]+/,/[A-Za-z][a-z]{2,}/)
;return i.contains.push({begin:h(/[ ]+/,"(",a,/[.]?[:]?([.][ ]|[ ])/,"){3}")}),i
},S=M("//","$"),k=M("/\\*","\\*/"),A=M("#","$");var R=Object.freeze({
__proto__:null,APOS_STRING_MODE:v,BACKSLASH_ESCAPE:x,BINARY_NUMBER_MODE:{
scope:"number",begin:y,relevance:0},BINARY_NUMBER_RE:y,COMMENT:M,
C_BLOCK_COMMENT_MODE:k,C_LINE_COMMENT_MODE:S,C_NUMBER_MODE:{scope:"number",
begin:w,relevance:0},C_NUMBER_RE:w,END_SAME_AS_BEGIN:e=>Object.assign(e,{
"on:begin":(e,n)=>{n.data._beginMatch=e[1]},"on:end":(e,n)=>{
n.data._beginMatch!==e[1]&&n.ignoreMatch()}}),HASH_COMMENT_MODE:A,IDENT_RE:_,
MATCH_NOTHING_RE:/\b\B/,METHOD_GUARD:{begin:"\\.\\s*"+E,relevance:0},
NUMBER_MODE:{scope:"number",begin:N,relevance:0},NUMBER_RE:N,
PHRASAL_WORDS_MODE:{
begin:/\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
},QUOTE_STRING_MODE:O,REGEXP_MODE:{scope:"regexp",begin:/\/(?=[^/\n]*\/)/,
end:/\/[gimuy]*/,contains:[x,{begin:/\[/,end:/\]/,relevance:0,contains:[x]}]},
RE_STARTERS_RE:"!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~",
SHEBANG:(e={})=>{const n=/^#![ ]*\//
;return e.binary&&(e.begin=h(n,/.*\b/,e.binary,/\b.*/)),s({scope:"meta",begin:n,
end:/$/,relevance:0,"on:begin":(e,n)=>{0!==e.index&&n.ignoreMatch()}},e)},
TITLE_MODE:{scope:"title",begin:_,relevance:0},UNDERSCORE_IDENT_RE:E,
UNDERSCORE_TITLE_MODE:{scope:"title",begin:E,relevance:0}});function T(e,n){
"."===e.input[e.index-1]&&n.ignoreMatch()}function I(e,n){
void 0!==e.className&&(e.scope=e.className,delete e.className)}function C(e,n){
n&&e.beginKeywords&&(e.begin="\\b("+e.beginKeywords.split(" ").join("|")+")(?!\\.)(?=\\b|\\s)",
e.__beforeBegin=T,e.keywords=e.keywords||e.beginKeywords,delete e.beginKeywords,
void 0===e.relevance&&(e.relevance=0))}function D(e,n){
Array.isArray(e.illegal)&&(e.illegal=b(...e.illegal))}function B(e,n){
if(e.match){
if(e.begin||e.end)throw Error("begin & end are not supported with match")
;e.begin=e.match,delete e.match}}function j(e,n){
void 0===e.relevance&&(e.relevance=1)}const L=(e,n)=>{if(!e.beforeMatch)return
;if(e.starts)throw Error("beforeMatch cannot be used with starts")
;const t=Object.assign({},e);Object.keys(e).forEach((n=>{delete e[n]
})),e.keywords=t.keywords,e.begin=h(t.beforeMatch,d(t.begin)),e.starts={
relevance:0,contains:[Object.assign(t,{endsParent:!0})]
},e.relevance=0,delete t.beforeMatch
},$=["of","and","for","in","not","or","if","then","parent","list","value"]
;function z(e,n,t="keyword"){const s=Object.create(null)
;return"string"==typeof e?i(t,e.split(" ")):Array.isArray(e)?i(t,e):Object.keys(e).forEach((t=>{
Object.assign(s,z(e[t],n,t))})),s;function i(e,t){
n&&(t=t.map((e=>e.toLowerCase()))),t.forEach((n=>{const t=n.split("|")
;s[t[0]]=[e,P(t[0],t[1])]}))}}function P(e,n){
return n?Number(n):(e=>$.includes(e.toLowerCase()))(e)?0:1}const H={},U=e=>{
console.error(e)},G=(e,...n)=>{console.log("WARN: "+e,...n)},W=(e,n)=>{
H[`${e}/${n}`]||(console.log(`Deprecated as of ${e}. ${n}`),H[`${e}/${n}`]=!0)
},K=Error();function F(e,n,{key:t}){let s=0;const i=e[t],a={},r={}
;for(let e=1;e<=n.length;e++)r[e+s]=i[e],a[e+s]=!0,s+=p(n[e-1])
;e[t]=r,e[t]._emit=a,e[t]._multi=!0}function Z(e){(e=>{
e.scope&&"object"==typeof e.scope&&null!==e.scope&&(e.beginScope=e.scope,
delete e.scope)})(e),"string"==typeof e.beginScope&&(e.beginScope={
_wrap:e.beginScope}),"string"==typeof e.endScope&&(e.endScope={_wrap:e.endScope
}),(e=>{if(Array.isArray(e.begin)){
if(e.skip||e.excludeBegin||e.returnBegin)throw U("skip, excludeBegin, returnBegin not compatible with beginScope: {}"),
K
;if("object"!=typeof e.beginScope||null===e.beginScope)throw U("beginScope must be object"),
K;F(e,e.begin,{key:"beginScope"}),e.begin=f(e.begin,{joinWith:""})}})(e),(e=>{
if(Array.isArray(e.end)){
if(e.skip||e.excludeEnd||e.returnEnd)throw U("skip, excludeEnd, returnEnd not compatible with endScope: {}"),
K
;if("object"!=typeof e.endScope||null===e.endScope)throw U("endScope must be object"),
K;F(e,e.end,{key:"endScope"}),e.end=f(e.end,{joinWith:""})}})(e)}function q(e){
function n(n,t){
return RegExp(l(n),"m"+(e.case_insensitive?"i":"")+(e.unicodeRegex?"u":"")+(t?"g":""))
}class t{constructor(){
this.matchIndexes={},this.regexes=[],this.matchAt=1,this.position=0}
addRule(e,n){
n.position=this.position++,this.matchIndexes[this.matchAt]=n,this.regexes.push([n,e]),
this.matchAt+=p(e)+1}compile(){0===this.regexes.length&&(this.exec=()=>null)
;const e=this.regexes.map((e=>e[1]));this.matcherRe=n(f(e,{joinWith:"|"
}),!0),this.lastIndex=0}exec(e){this.matcherRe.lastIndex=this.lastIndex
;const n=this.matcherRe.exec(e);if(!n)return null
;const t=n.findIndex(((e,n)=>n>0&&void 0!==e)),s=this.matchIndexes[t]
;return n.splice(0,t),Object.assign(n,s)}}class i{constructor(){
this.rules=[],this.multiRegexes=[],
this.count=0,this.lastIndex=0,this.regexIndex=0}getMatcher(e){
if(this.multiRegexes[e])return this.multiRegexes[e];const n=new t
;return this.rules.slice(e).forEach((([e,t])=>n.addRule(e,t))),
n.compile(),this.multiRegexes[e]=n,n}resumingScanAtSamePosition(){
return 0!==this.regexIndex}considerAll(){this.regexIndex=0}addRule(e,n){
this.rules.push([e,n]),"begin"===n.type&&this.count++}exec(e){
const n=this.getMatcher(this.regexIndex);n.lastIndex=this.lastIndex
;let t=n.exec(e)
;if(this.resumingScanAtSamePosition())if(t&&t.index===this.lastIndex);else{
const n=this.getMatcher(0);n.lastIndex=this.lastIndex+1,t=n.exec(e)}
return t&&(this.regexIndex+=t.position+1,
this.regexIndex===this.count&&this.considerAll()),t}}
if(e.compilerExtensions||(e.compilerExtensions=[]),
e.contains&&e.contains.includes("self"))throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.")
;return e.classNameAliases=s(e.classNameAliases||{}),function t(a,r){const o=a
;if(a.isCompiled)return o
;[I,B,Z,L].forEach((e=>e(a,r))),e.compilerExtensions.forEach((e=>e(a,r))),
a.__beforeBegin=null,[C,D,j].forEach((e=>e(a,r))),a.isCompiled=!0;let c=null
;return"object"==typeof a.keywords&&a.keywords.$pattern&&(a.keywords=Object.assign({},a.keywords),
c=a.keywords.$pattern,
delete a.keywords.$pattern),c=c||/\w+/,a.keywords&&(a.keywords=z(a.keywords,e.case_insensitive)),
o.keywordPatternRe=n(c,!0),
r&&(a.begin||(a.begin=/\B|\b/),o.beginRe=n(o.begin),a.end||a.endsWithParent||(a.end=/\B|\b/),
a.end&&(o.endRe=n(o.end)),
o.terminatorEnd=l(o.end)||"",a.endsWithParent&&r.terminatorEnd&&(o.terminatorEnd+=(a.end?"|":"")+r.terminatorEnd)),
a.illegal&&(o.illegalRe=n(a.illegal)),
a.contains||(a.contains=[]),a.contains=[].concat(...a.contains.map((e=>(e=>(e.variants&&!e.cachedVariants&&(e.cachedVariants=e.variants.map((n=>s(e,{
variants:null},n)))),e.cachedVariants?e.cachedVariants:Q(e)?s(e,{
starts:e.starts?s(e.starts):null
}):Object.isFrozen(e)?s(e):e))("self"===e?a:e)))),a.contains.forEach((e=>{t(e,o)
})),a.starts&&t(a.starts,r),o.matcher=(e=>{const n=new i
;return e.contains.forEach((e=>n.addRule(e.begin,{rule:e,type:"begin"
}))),e.terminatorEnd&&n.addRule(e.terminatorEnd,{type:"end"
}),e.illegal&&n.addRule(e.illegal,{type:"illegal"}),n})(o),o}(e)}function Q(e){
return!!e&&(e.endsWithParent||Q(e.starts))}class X extends Error{
constructor(e,n){super(e),this.name="HTMLInjectionError",this.html=n}}
const V=t,Y=s,J=Symbol("nomatch"),ee=t=>{
const s=Object.create(null),i=Object.create(null),a=[];let r=!0
;const o="Could not find the language '{}', did you forget to load/include a language module?",l={
disableAutodetect:!0,name:"Plain text",contains:[]};let p={
ignoreUnescapedHTML:!1,throwUnescapedHTML:!1,noHighlightRe:/^(no-?highlight)$/i,
languageDetectRe:/\blang(?:uage)?-([\w-]+)\b/i,classPrefix:"hljs-",
cssSelector:"pre code",languages:null,__emitter:c};function m(e){
return p.noHighlightRe.test(e)}function f(e,n,t){let s="",i=""
;"object"==typeof n?(s=e,
t=n.ignoreIllegals,i=n.language):(W("10.7.0","highlight(lang, code, ...args) has been deprecated."),
W("10.7.0","Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"),
i=e,s=n),void 0===t&&(t=!0);const a={code:s,language:i};M("before:highlight",a)
;const r=a.result?a.result:_(a.language,a.code,t)
;return r.code=a.code,M("after:highlight",r),r}function _(e,t,i,a){
const c=Object.create(null);function l(){if(!M.keywords)return void k.addText(A)
;let e=0;M.keywordPatternRe.lastIndex=0;let n=M.keywordPatternRe.exec(A),t=""
;for(;n;){t+=A.substring(e,n.index)
;const i=y.case_insensitive?n[0].toLowerCase():n[0],a=(s=i,M.keywords[s]);if(a){
const[e,s]=a
;if(k.addText(t),t="",c[i]=(c[i]||0)+1,c[i]<=7&&(R+=s),e.startsWith("_"))t+=n[0];else{
const t=y.classNameAliases[e]||e;g(n[0],t)}}else t+=n[0]
;e=M.keywordPatternRe.lastIndex,n=M.keywordPatternRe.exec(A)}var s
;t+=A.substring(e),k.addText(t)}function d(){null!=M.subLanguage?(()=>{
if(""===A)return;let e=null;if("string"==typeof M.subLanguage){
if(!s[M.subLanguage])return void k.addText(A)
;e=_(M.subLanguage,A,!0,S[M.subLanguage]),S[M.subLanguage]=e._top
}else e=E(A,M.subLanguage.length?M.subLanguage:null)
;M.relevance>0&&(R+=e.relevance),k.__addSublanguage(e._emitter,e.language)
})():l(),A=""}function g(e,n){
""!==e&&(k.startScope(n),k.addText(e),k.endScope())}function u(e,n){let t=1
;const s=n.length-1;for(;t<=s;){if(!e._emit[t]){t++;continue}
const s=y.classNameAliases[e[t]]||e[t],i=n[t];s?g(i,s):(A=i,l(),A=""),t++}}
function h(e,n){
return e.scope&&"string"==typeof e.scope&&k.openNode(y.classNameAliases[e.scope]||e.scope),
e.beginScope&&(e.beginScope._wrap?(g(A,y.classNameAliases[e.beginScope._wrap]||e.beginScope._wrap),
A=""):e.beginScope._multi&&(u(e.beginScope,n),A="")),M=Object.create(e,{parent:{
value:M}}),M}function b(e,t,s){let i=((e,n)=>{const t=e&&e.exec(n)
;return t&&0===t.index})(e.endRe,s);if(i){if(e["on:end"]){const s=new n(e)
;e["on:end"](t,s),s.isMatchIgnored&&(i=!1)}if(i){
for(;e.endsParent&&e.parent;)e=e.parent;return e}}
if(e.endsWithParent)return b(e.parent,t,s)}function m(e){
return 0===M.matcher.regexIndex?(A+=e[0],1):(C=!0,0)}function f(e){
const n=e[0],s=t.substring(e.index),i=b(M,e,s);if(!i)return J;const a=M
;M.endScope&&M.endScope._wrap?(d(),
g(n,M.endScope._wrap)):M.endScope&&M.endScope._multi?(d(),
u(M.endScope,e)):a.skip?A+=n:(a.returnEnd||a.excludeEnd||(A+=n),
d(),a.excludeEnd&&(A=n));do{
M.scope&&k.closeNode(),M.skip||M.subLanguage||(R+=M.relevance),M=M.parent
}while(M!==i.parent);return i.starts&&h(i.starts,e),a.returnEnd?0:n.length}
let N={};function w(s,a){const o=a&&a[0];if(A+=s,null==o)return d(),0
;if("begin"===N.type&&"end"===a.type&&N.index===a.index&&""===o){
if(A+=t.slice(a.index,a.index+1),!r){const n=Error(`0 width match regex (${e})`)
;throw n.languageName=e,n.badRule=N.rule,n}return 1}
if(N=a,"begin"===a.type)return(e=>{
const t=e[0],s=e.rule,i=new n(s),a=[s.__beforeBegin,s["on:begin"]]
;for(const n of a)if(n&&(n(e,i),i.isMatchIgnored))return m(t)
;return s.skip?A+=t:(s.excludeBegin&&(A+=t),
d(),s.returnBegin||s.excludeBegin||(A=t)),h(s,e),s.returnBegin?0:t.length})(a)
;if("illegal"===a.type&&!i){
const e=Error('Illegal lexeme "'+o+'" for mode "'+(M.scope||"<unnamed>")+'"')
;throw e.mode=M,e}if("end"===a.type){const e=f(a);if(e!==J)return e}
if("illegal"===a.type&&""===o)return a.index===t.length||(A+="\n"),1
;if(I>1e5&&I>3*a.index)throw Error("potential infinite loop, way more iterations than matches")
;return A+=o,o.length}const y=x(e)
;if(!y)throw U(o.replace("{}",e)),Error('Unknown language: "'+e+'"')
;const v=q(y);let O="",M=a||v;const S={},k=new p.__emitter(p);(()=>{const e=[]
;for(let n=M;n!==y;n=n.parent)n.scope&&e.unshift(n.scope)
;e.forEach((e=>k.openNode(e)))})();let A="",R=0,T=0,I=0,C=!1;try{
if(y.__emitTokens)y.__emitTokens(t,k);else{for(M.matcher.considerAll();;){
I++,C?C=!1:M.matcher.considerAll(),M.matcher.lastIndex=T
;const e=M.matcher.exec(t);if(!e)break;const n=w(t.substring(T,e.index),e)
;T=e.index+n}w(t.substring(T))}return k.finalize(),O=k.toHTML(),{language:e,
value:O,relevance:R,illegal:!1,_emitter:k,_top:M}}catch(n){
if(n.message&&n.message.includes("Illegal"))return{language:e,value:V(t),
illegal:!0,relevance:0,_illegalBy:{message:n.message,index:T,
context:t.slice(T-100,T+100),mode:n.mode,resultSoFar:O},_emitter:k};if(r)return{
language:e,value:V(t),illegal:!1,relevance:0,errorRaised:n,_emitter:k,_top:M}
;throw n}}function E(e,n){n=n||p.languages||Object.keys(s);const t=(e=>{
const n={value:V(e),illegal:!1,relevance:0,_top:l,_emitter:new p.__emitter(p)}
;return n._emitter.addText(e),n})(e),i=n.filter(x).filter(O).map((n=>_(n,e,!1)))
;i.unshift(t);const a=i.sort(((e,n)=>{
if(e.relevance!==n.relevance)return n.relevance-e.relevance
;if(e.language&&n.language){if(x(e.language).supersetOf===n.language)return 1
;if(x(n.language).supersetOf===e.language)return-1}return 0})),[r,o]=a,c=r
;return c.secondBest=o,c}function N(e){let n=null;const t=(e=>{
let n=e.className+" ";n+=e.parentNode?e.parentNode.className:""
;const t=p.languageDetectRe.exec(n);if(t){const n=x(t[1])
;return n||(G(o.replace("{}",t[1])),
G("Falling back to no-highlight mode for this block.",e)),n?t[1]:"no-highlight"}
return n.split(/\s+/).find((e=>m(e)||x(e)))})(e);if(m(t))return
;if(M("before:highlightElement",{el:e,language:t
}),e.dataset.highlighted)return void console.log("Element previously highlighted. To highlight again, first unset `dataset.highlighted`.",e)
;if(e.children.length>0&&(p.ignoreUnescapedHTML||(console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."),
console.warn("https://github.com/highlightjs/highlight.js/wiki/security"),
console.warn("The element with unescaped HTML:"),
console.warn(e)),p.throwUnescapedHTML))throw new X("One of your code blocks includes unescaped HTML.",e.innerHTML)
;n=e;const s=n.textContent,a=t?f(s,{language:t,ignoreIllegals:!0}):E(s)
;e.innerHTML=a.value,e.dataset.highlighted="yes",((e,n,t)=>{const s=n&&i[n]||t
;e.classList.add("hljs"),e.classList.add("language-"+s)
})(e,t,a.language),e.result={language:a.language,re:a.relevance,
relevance:a.relevance},a.secondBest&&(e.secondBest={
language:a.secondBest.language,relevance:a.secondBest.relevance
}),M("after:highlightElement",{el:e,result:a,text:s})}let w=!1;function y(){
if("loading"===document.readyState)return w||window.addEventListener("DOMContentLoaded",(()=>{
y()}),!1),void(w=!0);document.querySelectorAll(p.cssSelector).forEach(N)}
function x(e){return e=(e||"").toLowerCase(),s[e]||s[i[e]]}
function v(e,{languageName:n}){"string"==typeof e&&(e=[e]),e.forEach((e=>{
i[e.toLowerCase()]=n}))}function O(e){const n=x(e)
;return n&&!n.disableAutodetect}function M(e,n){const t=e;a.forEach((e=>{
e[t]&&e[t](n)}))}Object.assign(t,{highlight:f,highlightAuto:E,highlightAll:y,
highlightElement:N,
highlightBlock:e=>(W("10.7.0","highlightBlock will be removed entirely in v12.0"),
W("10.7.0","Please use highlightElement now."),N(e)),configure:e=>{p=Y(p,e)},
initHighlighting:()=>{
y(),W("10.6.0","initHighlighting() deprecated.  Use highlightAll() now.")},
initHighlightingOnLoad:()=>{
y(),W("10.6.0","initHighlightingOnLoad() deprecated.  Use highlightAll() now.")
},registerLanguage:(e,n)=>{let i=null;try{i=n(t)}catch(n){
if(U("Language definition for '{}' could not be registered.".replace("{}",e)),
!r)throw n;U(n),i=l}
i.name||(i.name=e),s[e]=i,i.rawDefinition=n.bind(null,t),i.aliases&&v(i.aliases,{
languageName:e})},unregisterLanguage:e=>{delete s[e]
;for(const n of Object.keys(i))i[n]===e&&delete i[n]},
listLanguages:()=>Object.keys(s),getLanguage:x,registerAliases:v,
autoDetection:O,inherit:Y,addPlugin:e=>{(e=>{
e["before:highlightBlock"]&&!e["before:highlightElement"]&&(e["before:highlightElement"]=n=>{
e["before:highlightBlock"](Object.assign({block:n.el},n))
}),e["after:highlightBlock"]&&!e["after:highlightElement"]&&(e["after:highlightElement"]=n=>{
e["after:highlightBlock"](Object.assign({block:n.el},n))})})(e),a.push(e)},
removePlugin:e=>{const n=a.indexOf(e);-1!==n&&a.splice(n,1)}}),t.debugMode=()=>{
r=!1},t.safeMode=()=>{r=!0},t.versionString="11.11.1",t.regex={concat:h,
lookahead:d,either:b,optional:u,anyNumberOfTimes:g}
;for(const n in R)"object"==typeof R[n]&&e(R[n]);return Object.assign(t,R),t
},ne=ee({});ne.newInstance=()=>ee({});const te={scope:"number",
match:"([-+]?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)|NaN|[-+]?Infinity",
relevance:0};var se=Object.freeze({__proto__:null,grmr_bash:e=>{
const n=e.regex,t={},s={begin:/\$\{/,end:/\}/,contains:["self",{begin:/:-/,
contains:[t]}]};Object.assign(t,{className:"variable",variants:[{
begin:n.concat(/\$[\w\d#@][\w\d_]*/,"(?![\\w\\d])(?![$])")},s]});const i={
className:"subst",begin:/\$\(/,end:/\)/,contains:[e.BACKSLASH_ESCAPE]
},a=e.inherit(e.COMMENT(),{match:[/(^|\s)/,/#.*$/],scope:{2:"comment"}}),r={
begin:/<<-?\s*(?=\w+)/,starts:{contains:[e.END_SAME_AS_BEGIN({begin:/(\w+)/,
end:/(\w+)/,className:"string"})]}},o={className:"string",begin:/"/,end:/"/,
contains:[e.BACKSLASH_ESCAPE,t,i]};i.contains.push(o);const c={begin:/\$?\(\(/,
end:/\)\)/,contains:[{begin:/\d+#[0-9a-f]+/,className:"number"},e.NUMBER_MODE,t]
},l=e.SHEBANG({binary:"(fish|bash|zsh|sh|csh|ksh|tcsh|dash|scsh)",relevance:10
}),d={className:"function",begin:/\w[\w\d_]*\s*\(\s*\)\s*\{/,returnBegin:!0,
contains:[e.inherit(e.TITLE_MODE,{begin:/\w[\w\d_]*/})],relevance:0};return{
name:"Bash",aliases:["sh","zsh"],keywords:{$pattern:/\b[a-z][a-z0-9._-]+\b/,
keyword:["if","then","else","elif","fi","time","for","while","until","in","do","done","case","esac","coproc","function","select"],
literal:["true","false"],
built_in:["break","cd","continue","eval","exec","exit","export","getopts","hash","pwd","readonly","return","shift","test","times","trap","umask","unset","alias","bind","builtin","caller","command","declare","echo","enable","help","let","local","logout","mapfile","printf","read","readarray","source","sudo","type","typeset","ulimit","unalias","set","shopt","autoload","bg","bindkey","bye","cap","chdir","clone","comparguments","compcall","compctl","compdescribe","compfiles","compgroups","compquote","comptags","comptry","compvalues","dirs","disable","disown","echotc","echoti","emulate","fc","fg","float","functions","getcap","getln","history","integer","jobs","kill","limit","log","noglob","popd","print","pushd","pushln","rehash","sched","setcap","setopt","stat","suspend","ttyctl","unfunction","unhash","unlimit","unsetopt","vared","wait","whence","where","which","zcompile","zformat","zftp","zle","zmodload","zparseopts","zprof","zpty","zregexparse","zsocket","zstyle","ztcp","chcon","chgrp","chown","chmod","cp","dd","df","dir","dircolors","ln","ls","mkdir","mkfifo","mknod","mktemp","mv","realpath","rm","rmdir","shred","sync","touch","truncate","vdir","b2sum","base32","base64","cat","cksum","comm","csplit","cut","expand","fmt","fold","head","join","md5sum","nl","numfmt","od","paste","ptx","pr","sha1sum","sha224sum","sha256sum","sha384sum","sha512sum","shuf","sort","split","sum","tac","tail","tr","tsort","unexpand","uniq","wc","arch","basename","chroot","date","dirname","du","echo","env","expr","factor","groups","hostid","id","link","logname","nice","nohup","nproc","pathchk","pinky","printenv","printf","pwd","readlink","runcon","seq","sleep","stat","stdbuf","stty","tee","test","timeout","tty","uname","unlink","uptime","users","who","whoami","yes"]
},contains:[l,e.SHEBANG(),d,c,a,r,{match:/(\/[a-z._-]+)+/},o,{match:/\\"/},{
className:"string",begin:/'/,end:/'/},{match:/\\'/},t]}},grmr_bnf:e=>({
name:"Backus\u2013Naur Form",contains:[{className:"attribute",begin:/</,end:/>/
},{begin:/::=/,end:/$/,contains:[{begin:/</,end:/>/
},e.C_LINE_COMMENT_MODE,e.C_BLOCK_COMMENT_MODE,e.APOS_STRING_MODE,e.QUOTE_STRING_MODE]
}]}),grmr_diff:e=>{const n=e.regex;return{name:"Diff",aliases:["patch"],
contains:[{className:"meta",relevance:10,
match:n.either(/^@@ +-\d+,\d+ +\+\d+,\d+ +@@/,/^@@ +-\d+ +\+\d+,\d+ +@@/,/^@@ +-\d+,\d+ +\+\d+ +@@/,/^@@ +-\d+ +\+\d+ +@@/,/^\*\*\* +\d+,\d+ +\*\*\*\*$/,/^--- +\d+,\d+ +----$/)
},{className:"comment",variants:[{
begin:n.either(/Index: /,/^index/,/={3,}/,/^-{3}/,/^\*{3} /,/^\+{3}/,/^diff --git/),
end:/$/},{match:/^\*{15}$/}]},{className:"addition",begin:/^\+/,end:/$/},{
className:"deletion",begin:/^-/,end:/$/},{className:"addition",begin:/^!/,
end:/$/}]}},grmr_dockerfile:e=>({name:"Dockerfile",aliases:["docker"],
case_insensitive:!0,
keywords:["from","maintainer","expose","env","arg","user","onbuild","stopsignal"],
contains:[e.HASH_COMMENT_MODE,e.APOS_STRING_MODE,e.QUOTE_STRING_MODE,e.NUMBER_MODE,{
beginKeywords:"run cmd entrypoint volume add copy workdir label healthcheck shell",
starts:{end:/[^\\]$/,subLanguage:"bash"}}],illegal:"</"}),grmr_ini:e=>{
const n=e.regex,t={className:"number",relevance:0,variants:[{
begin:/([+-]+)?[\d]+_[\d_]+/},{begin:e.NUMBER_RE}]},s=e.COMMENT();s.variants=[{
begin:/;/,end:/$/},{begin:/#/,end:/$/}];const i={className:"variable",
variants:[{begin:/\$[\w\d"][\w\d_]*/},{begin:/\$\{(.*?)\}/}]},a={
className:"literal",begin:/\bon|off|true|false|yes|no\b/},r={className:"string",
contains:[e.BACKSLASH_ESCAPE],variants:[{begin:"'''",end:"'''",relevance:10},{
begin:'"""',end:'"""',relevance:10},{begin:'"',end:'"'},{begin:"'",end:"'"}]
},o={begin:/\[/,end:/\]/,contains:[s,a,i,r,t,"self"],relevance:0
},c=n.either(/[A-Za-z0-9_-]+/,/"(\\"|[^"])*"/,/'[^']*'/);return{
name:"TOML, also INI",aliases:["toml"],case_insensitive:!0,illegal:/\S/,
contains:[s,{className:"section",begin:/\[+/,end:/\]+/},{
begin:n.concat(c,"(\\s*\\.\\s*",c,")*",n.lookahead(/\s*=\s*[^#\s]/)),
className:"attr",starts:{end:/$/,contains:[s,o,a,i,r,t]}}]}},grmr_json:e=>{
const n=["true","false","null"],t={scope:"literal",beginKeywords:n.join(" ")}
;return{name:"JSON",aliases:["jsonc","json5"],keywords:{literal:n},contains:[{
className:"attr",begin:/(("(\\.|[^\\"\r\n])*")|('(\\.|[^\\'\r\n])*'))(?=\s*:)/,
relevance:1.01},{match:/[{}[\],:]/,className:"punctuation",relevance:0
},e.APOS_STRING_MODE,e.QUOTE_STRING_MODE,t,te,e.C_LINE_COMMENT_MODE,e.C_BLOCK_COMMENT_MODE],
illegal:"\\S"}},grmr_plaintext:e=>({name:"Plain text",aliases:["text","txt"],
disableAutodetect:!0}),grmr_raptorfile:e=>({name:"raptorfile",
aliases:["raptor"],case_insensitive:!1,keywords:["FROM","ENV","MOUNT"],
contains:[e.HASH_COMMENT_MODE,e.APOS_STRING_MODE,e.QUOTE_STRING_MODE,e.NUMBER_MODE,{
beginKeywords:"RENDER WRITE MKDIR COPY INCLUDE RUN WORKDIR ENTRYPOINT CMD",
starts:{end:/[^\\]$/,subLanguage:"bash"}}],illegal:"</"}),grmr_ruby:e=>{
const n=e.regex,t="([a-zA-Z_]\\w*[!?=]?|[-+~]@|<<|>>|=~|===?|<=>|[<>]=?|\\*\\*|[-/+%^&*~`|]|\\[\\]=?)",s=n.either(/\b([A-Z]+[a-z0-9]+)+/,/\b([A-Z]+[a-z0-9]+)+[A-Z]+/),i=n.concat(s,/(::\w+)*/),a={
"variable.constant":["__FILE__","__LINE__","__ENCODING__"],
"variable.language":["self","super"],
keyword:["alias","and","begin","BEGIN","break","case","class","defined","do","else","elsif","end","END","ensure","for","if","in","module","next","not","or","redo","require","rescue","retry","return","then","undef","unless","until","when","while","yield","include","extend","prepend","public","private","protected","raise","throw"],
built_in:["proc","lambda","attr_accessor","attr_reader","attr_writer","define_method","private_constant","module_function"],
literal:["true","false","nil"]},r={className:"doctag",begin:"@[A-Za-z]+"},o={
begin:"#<",end:">"},c=[e.COMMENT("#","$",{contains:[r]
}),e.COMMENT("^=begin","^=end",{contains:[r],relevance:10
}),e.COMMENT("^__END__",e.MATCH_NOTHING_RE)],l={className:"subst",begin:/#\{/,
end:/\}/,keywords:a},d={className:"string",contains:[e.BACKSLASH_ESCAPE,l],
variants:[{begin:/'/,end:/'/},{begin:/"/,end:/"/},{begin:/`/,end:/`/},{
begin:/%[qQwWx]?\(/,end:/\)/},{begin:/%[qQwWx]?\[/,end:/\]/},{
begin:/%[qQwWx]?\{/,end:/\}/},{begin:/%[qQwWx]?</,end:/>/},{begin:/%[qQwWx]?\//,
end:/\//},{begin:/%[qQwWx]?%/,end:/%/},{begin:/%[qQwWx]?-/,end:/-/},{
begin:/%[qQwWx]?\|/,end:/\|/},{begin:/\B\?(\\\d{1,3})/},{
begin:/\B\?(\\x[A-Fa-f0-9]{1,2})/},{begin:/\B\?(\\u\{?[A-Fa-f0-9]{1,6}\}?)/},{
begin:/\B\?(\\M-\\C-|\\M-\\c|\\c\\M-|\\M-|\\C-\\M-)[\x20-\x7e]/},{
begin:/\B\?\\(c|C-)[\x20-\x7e]/},{begin:/\B\?\\?\S/},{
begin:n.concat(/<<[-~]?'?/,n.lookahead(/(\w+)(?=\W)[^\n]*\n(?:[^\n]*\n)*?\s*\1\b/)),
contains:[e.END_SAME_AS_BEGIN({begin:/(\w+)/,end:/(\w+)/,
contains:[e.BACKSLASH_ESCAPE,l]})]}]},g="[0-9](_?[0-9])*",u={className:"number",
relevance:0,variants:[{
begin:`\\b([1-9](_?[0-9])*|0)(\\.(${g}))?([eE][+-]?(${g})|r)?i?\\b`},{
begin:"\\b0[dD][0-9](_?[0-9])*r?i?\\b"},{begin:"\\b0[bB][0-1](_?[0-1])*r?i?\\b"
},{begin:"\\b0[oO][0-7](_?[0-7])*r?i?\\b"},{
begin:"\\b0[xX][0-9a-fA-F](_?[0-9a-fA-F])*r?i?\\b"},{
begin:"\\b0(_?[0-7])+r?i?\\b"}]},h={variants:[{match:/\(\)/},{
className:"params",begin:/\(/,end:/(?=\))/,excludeBegin:!0,endsParent:!0,
keywords:a}]},b=[d,{variants:[{match:[/class\s+/,i,/\s+<\s+/,i]},{
match:[/\b(class|module)\s+/,i]}],scope:{2:"title.class",
4:"title.class.inherited"},keywords:a},{match:[/(include|extend)\s+/,i],scope:{
2:"title.class"},keywords:a},{relevance:0,match:[i,/\.new[. (]/],scope:{
1:"title.class"}},{relevance:0,match:/\b[A-Z][A-Z_0-9]+\b/,
className:"variable.constant"},{relevance:0,match:s,scope:"title.class"},{
match:[/def/,/\s+/,t],scope:{1:"keyword",3:"title.function"},contains:[h]},{
begin:e.IDENT_RE+"::"},{className:"symbol",
begin:e.UNDERSCORE_IDENT_RE+"(!|\\?)?:",relevance:0},{className:"symbol",
begin:":(?!\\s)",contains:[d,{begin:t}],relevance:0},u,{className:"variable",
begin:"(\\$\\W)|((\\$|@@?)(\\w+))(?=[^@$?])(?![A-Za-z])(?![@$?'])"},{
className:"params",begin:/\|(?!=)/,end:/\|/,excludeBegin:!0,excludeEnd:!0,
relevance:0,keywords:a},{begin:"("+e.RE_STARTERS_RE+"|unless)\\s*",
keywords:"unless",contains:[{className:"regexp",contains:[e.BACKSLASH_ESCAPE,l],
illegal:/\n/,variants:[{begin:"/",end:"/[a-z]*"},{begin:/%r\{/,end:/\}[a-z]*/},{
begin:"%r\\(",end:"\\)[a-z]*"},{begin:"%r!",end:"![a-z]*"},{begin:"%r\\[",
end:"\\][a-z]*"}]}].concat(o,c),relevance:0}].concat(o,c)
;l.contains=b,h.contains=b;const p=[{begin:/^\s*=>/,starts:{end:"$",contains:b}
},{className:"meta.prompt",
begin:"^([>?]>|[\\w#]+\\(\\w+\\):\\d+:\\d+[>*]|(\\w+-)?\\d+\\.\\d+\\.\\d+(p\\d+)?[^\\d][^>]+>)(?=[ ])",
starts:{end:"$",keywords:a,contains:b}}];return c.unshift(o),{name:"Ruby",
aliases:["rb","gemspec","podspec","thor","irb"],keywords:a,illegal:/\/\*/,
contains:[e.SHEBANG({binary:"ruby"})].concat(p).concat(c).concat(b)}},
grmr_rust:e=>{
const n=e.regex,t=/(r#)?/,s=n.concat(t,e.UNDERSCORE_IDENT_RE),i=n.concat(t,e.IDENT_RE),a={
className:"title.function.invoke",relevance:0,
begin:n.concat(/\b/,/(?!let|for|while|if|else|match\b)/,i,n.lookahead(/\s*\(/))
},r="([ui](8|16|32|64|128|size)|f(32|64))?",o=["drop ","Copy","Send","Sized","Sync","Drop","Fn","FnMut","FnOnce","ToOwned","Clone","Debug","PartialEq","PartialOrd","Eq","Ord","AsRef","AsMut","Into","From","Default","Iterator","Extend","IntoIterator","DoubleEndedIterator","ExactSizeIterator","SliceConcatExt","ToString","assert!","assert_eq!","bitflags!","bytes!","cfg!","col!","concat!","concat_idents!","debug_assert!","debug_assert_eq!","env!","eprintln!","panic!","file!","format!","format_args!","include_bytes!","include_str!","line!","local_data_key!","module_path!","option_env!","print!","println!","select!","stringify!","try!","unimplemented!","unreachable!","vec!","write!","writeln!","macro_rules!","assert_ne!","debug_assert_ne!"],c=["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","f32","f64","str","char","bool","Box","Option","Result","String","Vec"]
;return{name:"Rust",aliases:["rs"],keywords:{$pattern:e.IDENT_RE+"!?",type:c,
keyword:["abstract","as","async","await","become","box","break","const","continue","crate","do","dyn","else","enum","extern","false","final","fn","for","if","impl","in","let","loop","macro","match","mod","move","mut","override","priv","pub","ref","return","self","Self","static","struct","super","trait","true","try","type","typeof","union","unsafe","unsized","use","virtual","where","while","yield"],
literal:["true","false","Some","None","Ok","Err"],built_in:o},illegal:"</",
contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{contains:["self"]
}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"symbol",begin:/'[a-zA-Z_][a-zA-Z0-9_]*(?!')/},{scope:"string",
variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{begin:/b?'/,end:/'/,contains:[{
scope:"char.escape",match:/\\('|\w|x\w{2}|u\w{4}|U\w{8})/}]}]},{
className:"number",variants:[{begin:"\\b0b([01_]+)"+r},{begin:"\\b0o([0-7_]+)"+r
},{begin:"\\b0x([A-Fa-f0-9_]+)"+r},{
begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+r}],relevance:0},{
begin:[/fn/,/\s+/,s],className:{1:"keyword",3:"title.function"}},{
className:"meta",begin:"#!?\\[",end:"\\]",contains:[{className:"string",
begin:/"/,end:/"/,contains:[e.BACKSLASH_ESCAPE]}]},{
begin:[/let/,/\s+/,/(?:mut\s+)?/,s],className:{1:"keyword",3:"keyword",
4:"variable"}},{begin:[/for/,/\s+/,s,/\s+/,/in/],className:{1:"keyword",
3:"variable",5:"keyword"}},{begin:[/type/,/\s+/,s],className:{1:"keyword",
3:"title.class"}},{begin:[/(?:trait|enum|struct|union|impl|for)/,/\s+/,s],
className:{1:"keyword",3:"title.class"}},{begin:e.IDENT_RE+"::",keywords:{
keyword:"Self",built_in:o,type:c}},{className:"punctuation",begin:"->"},a]}},
grmr_yaml:e=>{
const n="true false yes no null",t="[\\w#;/?:@&=+$,.~*'()[\\]]+",s={
className:"string",relevance:0,variants:[{begin:/"/,end:/"/},{begin:/\S+/}],
contains:[e.BACKSLASH_ESCAPE,{className:"template-variable",variants:[{
begin:/\{\{/,end:/\}\}/},{begin:/%\{/,end:/\}/}]}]},i=e.inherit(s,{variants:[{
begin:/'/,end:/'/,contains:[{begin:/''/,relevance:0}]},{begin:/"/,end:/"/},{
begin:/[^\s,{}[\]]+/}]}),a={end:",",endsWithParent:!0,excludeEnd:!0,keywords:n,
relevance:0},r={begin:/\{/,end:/\}/,contains:[a],illegal:"\\n",relevance:0},o={
begin:"\\[",end:"\\]",contains:[a],illegal:"\\n",relevance:0},c=[{
className:"attr",variants:[{begin:/[\w*@][\w*@ :()\./-]*:(?=[ \t]|$)/},{
begin:/"[\w*@][\w*@ :()\./-]*":(?=[ \t]|$)/},{
begin:/'[\w*@][\w*@ :()\./-]*':(?=[ \t]|$)/}]},{className:"meta",
begin:"^---\\s*$",relevance:10},{className:"string",
begin:"[\\|>]([1-9]?[+-])?[ ]*\\n( +)[^ ][^\\n]*\\n(\\2[^\\n]+\\n?)*"},{
begin:"<%[%=-]?",end:"[%-]?%>",subLanguage:"ruby",excludeBegin:!0,excludeEnd:!0,
relevance:0},{className:"type",begin:"!\\w+!"+t},{className:"type",
begin:"!<"+t+">"},{className:"type",begin:"!"+t},{className:"type",begin:"!!"+t
},{className:"meta",begin:"&"+e.UNDERSCORE_IDENT_RE+"$"},{className:"meta",
begin:"\\*"+e.UNDERSCORE_IDENT_RE+"$"},{className:"bullet",begin:"-(?=[ ]|$)",
relevance:0},e.HASH_COMMENT_MODE,{beginKeywords:n,keywords:{literal:n}},{
className:"number",
begin:"\\b[0-9]{4}(-[0-9][0-9]){0,2}([Tt \\t][0-9][0-9]?(:[0-9][0-9]){2})?(\\.[0-9]*)?([ \\t])*(Z|[-+][0-9][0-9]?(:[0-9][0-9])?)?\\b"
},{className:"number",begin:e.C_NUMBER_RE+"\\b",relevance:0},r,o,{
className:"string",relevance:0,begin:/'/,end:/'/,contains:[{match:/''/,
scope:"char.escape",relevance:0}]},s],l=[...c]
;return l.pop(),l.push(i),a.contains=l,{name:"YAML",case_insensitive:!0,
aliases:["yml"],contains:c}}});const ie=ne;for(const e of Object.keys(se)){
const n=e.replace("grmr_","").replace("_","-");ie.registerLanguage(n,se[e])}
return ie}()
;"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=hljs);