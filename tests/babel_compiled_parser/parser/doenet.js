// This file was generated by lezer-generator. You probably shouldn't edit it.
import { LRParser } from "@lezer/lr";
import { startTag, commentContent, elementContext } from "./tokens.js";
import { NodeProp } from "@lezer/common";
export const parser = LRParser.deserialize({
  version: 13,
  states: "*[OQOTOOO]OYO'#CdOeO`O'#CfO!OOTO'#CeOOOP'#Ce'#CeOOOP'#Cx'#CxOOOP'#Co'#CoQQOTOOOOOQ'#Cp'#CpO!VOYO,59OOOOP,59O,59OO!_OpO,59QO!mO`O'#ClOOOP'#DT'#DTOOOP'#Cu'#CuO!rOTO,59PO!yO`O'#CmOOOP,59P,59POOOP-E6m-E6mOOOQ-E6n-E6nOOOP1G.j1G.jOOOO'#Cq'#CqO#ROpO1G.lO#aOpO'#ChOOOO'#Cr'#CrO#rOpO1G.lOOOP1G.l1G.lOOOP1G.t1G.tO#}OWO,59WOOOP-E6s-E6sOOOP1G.k1G.kO$SO`O,59XO$[OWO,59XOOOO-E6o-E6oO$dOpO7+$WOOOP7+$W7+$WOOOP7+$`7+$`O$oOpO,59SO%QOWO,59SOOOO-E6p-E6pOOOP1G.r1G.rO%]OWO1G.sO%]OWO1G.sOOOP1G.s1G.sOOOP<<Gr<<GrOOOP<<Gz<<GzO%QOWO1G.nO%QOWO1G.nO%eO!bO'#C}O%mO#tO'#DQOOOO'#Ck'#CkO%uOpO1G.nO&TOWO7+$_OOOP7+$_7+$_O%QOWO7+$YO&]OpO7+$YOOOO'#Cs'#CsO&kO!bO,59iOOOO,59i,59iOOOO'#Ct'#CtO&sO#tO,59lOOOO,59l,59lO&]OpO7+$YOOOP<<Gy<<GyO&{OpO<<GtO&{OpO<<GtOOOO-E6q-E6qOOOO1G/T1G/TOOOO-E6r-E6rOOOO1G/W1G/WO'ZOpOAN=`",
  stateData: "'i~OPQOVTOnPO~OkWOmYO~OZZO~OPQOQ`OS[OT]OV]OnPO~ORaO~PjOkWOmdO~O]gOojOpeOxkO~OZlO~ORnO~PjOZpOpeO~O]gOosOpeOxtO~O^vOpeO][Xo[Xx[X~O]gOosOxtO~OoxO~OZyOpeO~Oo{OpeO~O]gOo|Ox}O~O^!OOpeO][ao[ax[a~OpeOr!QOu!RO~Oo!VOpeO~Or![Os!YO~Ou!_Ov!]O~OpeO][io[ix[i~Oo!aOpeO~OpeO][qo[qx[q~Or!eOs!YO~Ou!gOv!]O~OpeO][yo[yx[y~OpeO][!Ro[!Rx[!R~O",
  goto: "$yxPPPPPPPPyy!RP!XPP!_}!i!o!u!{#R#}$X$_$ePP$kPPPP$oPP$oPP$uSTOVT]R_XRORV_XhZfirQ!TvS!X!O!PR!b!WQaRRn_XSORV_QVORbVQXPRcXQfZQo`dqfouz!P!U!W!`!c!hQugQzpQ!PvQ!UyQ!W!OQ!`!TQ!c!XR!h!bQiZQrfTwirQ!Z!QR!d!ZQ!^!RR!f!^Q_RRm_TUOVX!Sv!O!P!WT^R_",
  nodeNames: "⚠ StartTag StartCloseTag MissingCloseTag StartCloseTag StartCloseTag Document Text Comment Element OpenTag TagName Attribute AttributeName Is AttributeValue MismatchedCloseTag CloseTag SelfClosingTag",
  maxTerm: 40,
  context: elementContext,
  nodeProps: [[NodeProp.closedBy, 1, "SelfCloseEndTag EndTag", 10, "CloseTag MissingCloseTag"], [NodeProp.openedBy, 17, "OpenTag"]],
  skippedNodes: [0],
  repeatNodeCount: 7,
  tokenData: "6e!aRxOX#oX^({^p#opq({qr#ors*{sw#owx+ex}#o}!O+}!O!P#o!P!Q0}!Q![.y![!^#o!^!_2e!_!`5R!`!a5s!a!c#o!c!}.y!}#R#o#R#S.y#S#T#o#T#o.y#o#y#o#y#z({#z$f#o$f$g({$g#BY#o#BY#BZ({#BZ$IS#o$IS$I_({$I_$I|#o$I|$JO({$JO$JT#o$JT$JU({$JU$KV#o$KV$KW({$KW&FU#o&FU&FV({&FV~#o!R#xVVPvps`Or#ors$_sw#owx&Zx!^#o!^!_'k!_~#oa$fTVPs`Ow$_wx$ux!^$_!^!_%a!_~$_P$zRVPO!^$u!^!_%T!_~$uP%WRYZ$upq$u!_!`$ua%fWs`OY&OYZ$_Zp&Opq$_qw&Ox!_&O!_!`$_!`~&O`&TQs`Ow&Ox~&Oq&bTVPvpOr&Zrs$us!^&Z!^!_&q!_~&Zq&vWvpOY'`YZ&ZZp'`pq&Zqr'`s!_'`!_!`&Z!`~'`p'eQvpOr'`s~'`!R'rZvps`OY(eYZ#oZp(epq#oqr(ers&Osw(ewx'`x!_(e!_!`#o!`~(e!Q(lTvps`Or(ers&Osw(ewx'`x~(e!a)WkVPvps`p^OX#oX^({^p#opq({qr#ors$_sw#owx&Zx!^#o!^!_'k!_#y#o#y#z({#z$f#o$f$g({$g#BY#o#BY#BZ({#BZ$IS#o$IS$I_({$I_$I|#o$I|$JO({$JO$JT#o$JT$JU({$JU$KV#o$KV$KW({$KW&FU#o&FU&FV({&FV~#o!T+UTurVPs`Ow$_wx$ux!^$_!^!_%a!_~$_!T+nTrbVPvpOr&Zrs$us!^&Z!^!_&q!_~&Z!a,[a]WZSVPvps`Or#ors$_sw#owx&Zx}#o}!O-a!O!Q#o!Q![.y![!^#o!^!_'k!_!c#o!c!}.y!}#R#o#R#S.y#S#T#o#T#o.y#o~#o!a-nc]WZSVPvps`Or#ors$_sw#owx&Zx}#o}!O.y!O!Q#o!Q![.y![!^#o!^!_'k!_!`#o!`!a0]!a!c#o!c!}.y!}#R#o#R#S.y#S#T#o#T#o.y#o~#o!_/Wa]WZSVPvps`Or#ors$_sw#owx&Zx}#o}!O.y!O!Q#o!Q![.y![!^#o!^!_'k!_!c#o!c!}.y!}#R#o#R#S.y#S#T#o#T#o.y#o~#o!T0hVmQVPvps`Or#ors$_sw#owx&Zx!^#o!^!_'k!_~#o!Z1WXVPvps`Or#ors$_sw#owx&Zx!^#o!^!_'k!_!`#o!`!a1s!a~#o!Z2OVVPvps`xWOr#ors$_sw#owx&Zx!^#o!^!_'k!_~#o!R2lZvps`OY(eYZ#oZp(epq#oqr3_rs&Osw(ewx'`x!_(e!_!`#o!`~(e!R3fVvps`Or(ers&Osw(ewx'`x}(e}!O3{!O~(e!R4SVvps`Or(ers&Osw(ewx'`x}(e}!O4i!O~(e!R4rTvps`nPOr(ers&Osw(ewx'`x~(e!Z5^V^WVPvps`Or#ors$_sw#owx&Zx!^#o!^!_'k!_~#o!]6OVVPvps`oYOr#ors$_sw#owx&Zx!^#o!^!_'k!_~#o",
  tokenizers: [startTag, commentContent, 0, 1, 2, 3, 4, 5],
  topRules: {
    "Document": [0, 6]
  },
  tokenPrec: 0
});