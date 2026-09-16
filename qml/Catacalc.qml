import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

// The approved artwork is a texture atlas, with all labels removed.
// Every label, expression, result, history entry and hit target below is live.
Item {
 id: garden
 required property var host
 property alias expressionEditor: expression
 property real designWidth: host.historyOpen ? 1333 : 899
 property real designHeight: 1180 + extraHeight
 property real extraHeight: host.scientific ? 210 : 0
 property real uiScale: Math.min(width / designWidth, height / designHeight)
 signal optionsRequested()
 signal resultMenuRequested()
 signal clearHistoryRequested()
 function focusCalculator() { garden.forceActiveFocus() }
 function calculate(key) { host.act(key); focusCalculator() }
 function editExpression() { expression.forceActiveFocus(); expression.selectAll() }
 function screenExpression(raw) { return host.expressionText(raw.replace(/\*/g,"×").replace(/\//g,"÷").replace(/-/g,"−")) }
 focus: visible
 Keys.onPressed: event => {
  if(event.modifiers & (Qt.ControlModifier|Qt.AltModifier|Qt.MetaModifier)) return
  if(event.key===Qt.Key_Return||event.key===Qt.Key_Enter||event.key===Qt.Key_Equal){calculate("=");event.accepted=true}
  else if(event.key===Qt.Key_Backspace){calculate("back");event.accepted=true}
  else if(event.key===Qt.Key_Delete){calculate("CE");event.accepted=true}
  else if(event.text && event.text.length===1 && "0123456789.,+-*/()%^!".includes(event.text)){calculate(({"-":"−","*":"×","/":"÷",",":"."})[event.text]||event.text);event.accepted=true}
 }
 component Patch: Image {
  property rect region: Qt.rect(0,0,1333,1180)
  source: "assets/catacalc-skin.png"
  sourceSize: Qt.size(1333,1180)
  sourceClipRect: region
  width: region.width; height: region.height
  smooth: true
 }
 component Hit: Button {
  id: hit
  property int labelSize: 42
  property color ink: "#0b3520"
  property bool bold: false
  property string tone: "cream"
  property string hint: text
  hoverEnabled: true
  padding: 0
  background: Rectangle {
   id:disk
   anchors.centerIn:parent;width:Math.min(hit.width,hit.height);height:width;radius:width/2
   property color topColor:hit.tone==="green"?"#a7e137":hit.tone==="gold"?"#ffe477":hit.tone==="sage"?"#dbe6c4":hit.tone==="header"?"#326348":hit.tone==="plain"?"#e9efd8":"#fffef3"
   property color bottomColor:hit.tone==="green"?"#77b91c":hit.tone==="gold"?"#ffc743":hit.tone==="sage"?"#becfa1":hit.tone==="header"?"#17432f":hit.tone==="plain"?"#dce6c6":"#eeebd8"
   gradient:Gradient{GradientStop{position:0;color:disk.topColor}GradientStop{position:1;color:disk.bottomColor}}
   border.width:hit.visualFocus?3:2;border.color:hit.visualFocus?"#477c2c":hit.tone==="header"?"#568268":"#d6dfba"
   Rectangle{z:-1;x:0;y:4;width:parent.width;height:parent.height;radius:width/2;color:"#28435a26"}
   Rectangle{anchors.fill:parent;radius:width/2;color:hit.down?"#350b3520":hit.hovered?"#33ffffff":"transparent"}
  }
  contentItem: Text { text:hit.text; color:hit.ink; opacity:hit.enabled?1:0.48; font.family:"DejaVu Sans"; font.pixelSize:hit.labelSize; font.weight:hit.bold?Font.Bold:Font.Normal; horizontalAlignment:Text.AlignHCenter; verticalAlignment:Text.AlignVCenter }
  Accessible.name: hint
  ToolTip.visible: hovered; ToolTip.delay: 700; ToolTip.text: hint
 }
 Item {
  id: design
  width: garden.designWidth; height:garden.designHeight
  scale: garden.uiScale; transformOrigin:Item.TopLeft
  x:(garden.width-width*scale)/2; y:(garden.height-height*scale)/2
  // The split below inserts science keys without stretching the mascot or keys.
  Patch { region:Qt.rect(0,0,855,520) }
  Patch { y:520+garden.extraHeight; region:Qt.rect(0,520,855,660) }
  Rectangle { visible:host.scientific; x:38;y:520;width:817;height:garden.extraHeight; color:"#d4dfb7" }
  Patch { visible:host.scientific;y:520;region:Qt.rect(0,442,59,68);height:garden.extraHeight }
  Item { visible:host.historyOpen; x:855
   Patch { region:Qt.rect(855,0,478,392) }
   Patch { visible:!host.scientific;y:392; region:Qt.rect(855,392,478,388) }
   Patch { visible:host.scientific;y:392;region:Qt.rect(855,670,478,100);height:388+garden.extraHeight }
   Patch { y:780+garden.extraHeight; region:Qt.rect(855,780,478,400) }
  }
  Patch { visible:!host.historyOpen; x:855;region:Qt.rect(1289,0,44,1180);height:garden.designHeight }
  Patch { visible:!host.historyOpen;region:Qt.rect(0,0,1333,218);width:899;height:218 }
  Rectangle { x:55;y:435;width:786;height:680+garden.extraHeight;radius:12
   gradient:Gradient{GradientStop{position:0;color:"#dce5c8"}GradientStop{position:1;color:"#d0deb5"}}
  }
  MouseArea { x:40;y:48;width:host.historyOpen?1252:815;height:82;onPressed:host.startSystemMove();onDoubleClicked:host.visibility===Window.Maximized?host.showNormal():host.showMaximized() }
  Hit { x:host.historyOpen?60:40;y:59;width:host.historyOpen?57:38;height:63;text:"☰";tone:"header";labelSize:host.historyOpen?39:28;ink:"#fffced";hint:"Options";onClicked:garden.optionsRequested() }
  // Native vector leaf mark, matching the small leaf emblem in the mockup.
  Item { x:host.historyOpen?145:97;y:69;width:47;height:47;scale:host.historyOpen?1:0.7;transformOrigin:Item.TopLeft
   Rectangle { x:27;y:18;width:3;height:28;rotation:21;color:"#9bd748" }
   Rectangle { x:25;y:1;width:19;height:29;radius:14;rotation:33;color:"#9dd948" }
   Rectangle { x:4;y:15;width:16;height:24;radius:12;rotation:-40;color:"#99d63d" }
  }
  Text { x:host.historyOpen?204:137;y:host.historyOpen?67:73;text:"Catercalc";color:"#fffdf1";font.family:"DejaVu Sans";font.pixelSize:host.historyOpen?40:29;font.bold:true }
  Patch { x:host.historyOpen?948:638;y:56;region:Qt.rect(1040,56,82,68);width:host.historyOpen?82:57;height:68 }
  Row { x:host.historyOpen?950:640;y:58;spacing:host.historyOpen?9:6
   Hit { width:host.historyOpen?78:53;height:64;text:"◷";tone:"gold";labelSize:host.historyOpen?43:31;ink:"#123d25";hint:"Show / hide history · Ctrl+H";onClicked:host.toggleHistory() }
   Hit { width:host.historyOpen?74:50;height:64;text:"−";tone:"header";labelSize:host.historyOpen?43:31;ink:"#fffced";hint:"Minimize";onClicked:host.showMinimized() }
   Hit { width:host.historyOpen?74:50;height:64;text:"□";tone:"header";labelSize:host.historyOpen?43:31;ink:"#fffced";hint:"Maximize / restore";onClicked:host.visibility===Window.Maximized?host.showNormal():host.showMaximized() }
   Hit { width:host.historyOpen?74:50;height:64;text:"×";tone:"header";labelSize:host.historyOpen?43:31;ink:"#fffced";hint:"Close";onClicked:host.close() }
  }
  TextField {
   id:expression;x:96;y:236;width:707;height:57;padding:0
   text:activeFocus?host.calcState.expression:garden.screenExpression(host.calcState.expression)
   color:"#0e3821";font.family:"DejaVu Sans";font.pixelSize:36;font.weight:Font.DemiBold
   placeholderText:"Enter a calculation";placeholderTextColor:"#708261";selectByMouse:true
   background:Rectangle{radius:8;color:expression.activeFocus?"#50ffffff":"transparent";border.width:expression.activeFocus?2:0;border.color:"#91b465"}
   onTextEdited:host.act("edit:"+host.normalize(text))
   Keys.onReturnPressed:event=>{event.accepted=true;garden.calculate("=")}
   Keys.onEnterPressed:event=>{event.accepted=true;garden.calculate("=")}
   Accessible.name:"Expression"
  }
  Text { x:68;y:304;text:host.calcState.memory!==null?"M":"";font.pixelSize:18;color:"#58782c" }
  Text {
   x:99;y:293;width:696;height:110;text:host.pretty(host.calcState.result)
   color:host.calcState.error?"#a33722":"#082f1b";font.family:"DejaVu Sans";font.pixelSize:host.calcState.error?34:96;font.weight:Font.Bold
   minimumPixelSize:24;fontSizeMode:Text.Fit;horizontalAlignment:Text.AlignRight;verticalAlignment:Text.AlignVCenter;maximumLineCount:1
   MouseArea{anchors.fill:parent;acceptedButtons:Qt.RightButton;onClicked:garden.resultMenuRequested()}
  }
  Repeater {
   model:["MC","MR","MS","M+","M−","fx"]
   delegate:Hit {
    required property string modelData;required property int index
    x:62+index*130;y:444;width:122;height:64;text:modelData;tone:"sage";labelSize:index===5?38:28
    hint:(["Clear memory","Recall memory","Store result","Add to memory","Subtract from memory","Scientific mode · Ctrl+Shift+S"])[index]
    // Memory clear/recall are harmless no-ops when empty; keep the original ink.
    onClicked:index===5?host.toggleScientific():garden.calculate(modelData==="M−"?"M-":modelData)
   }
  }
  GridLayout {
   visible:host.scientific;x:61;y:522;width:772;height:200;columns:5;rowSpacing:8;columnSpacing:9
   Repeater { model:[{t:host.degrees?"DEG":"RAD",k:"angle"},{t:"(",k:"("},{t:")",k:")"},{t:"π",k:"pi"},{t:"e",k:"e"},{t:"sin",k:"sin"},{t:"cos",k:"cos"},{t:"tan",k:"tan"},{t:"ln",k:"ln"},{t:"log",k:"log"},{t:"xʸ",k:"^"},{t:"x!",k:"!"},{t:"|x|",k:"abs"},{t:"↶",k:"undo"},{t:"↷",k:"redo"}]
    delegate:Hit { required property var modelData;text:modelData.t;labelSize:28;Layout.fillWidth:true;Layout.fillHeight:true
     onClicked:{if(modelData.k==="angle")host.toggleAngle();else garden.calculate(modelData.k)}
    }
   }
  }
  Repeater {
   model:[{t:"%",k:"%"},{t:"±",k:"±"},{t:"√",k:"sqrt"},{t:"x²",k:"square"},{t:"1/x",k:"reciprocal"},{t:"CE",k:"CE"}]
   delegate:Hit { required property var modelData;required property int index;x:61+index*130;y:522+garden.extraHeight;width:122;height:77;text:modelData.t;labelSize:38;onClicked:garden.calculate(modelData.k) }
  }
  Repeater {
   model:[{t:"C",k:"C",r:0,c:0},{t:"⌫",k:"back",r:0,c:1},{t:"÷",k:"÷",r:0,c:2},{t:"×",k:"×",r:0,c:3},{t:"7",k:"7",r:1,c:0},{t:"8",k:"8",r:1,c:1},{t:"9",k:"9",r:1,c:2},{t:"−",k:"−",r:1,c:3},{t:"4",k:"4",r:2,c:0},{t:"5",k:"5",r:2,c:1},{t:"6",k:"6",r:2,c:2},{t:"+",k:"+",r:2,c:3},{t:"1",k:"1",r:3,c:0},{t:"2",k:"2",r:3,c:1},{t:"3",k:"3",r:3,c:2},{t:"=",k:"=",r:3,c:3,rows:2},{t:"0",k:"0",r:4,c:0,cols:2},{t:".",k:".",r:4,c:2}]
   delegate:Hit { required property var modelData;x:61+modelData.c*196;y:611+modelData.r*100+garden.extraHeight;width:modelData.cols?381:183;height:modelData.rows?196:91;text:modelData.k==="back"?"":modelData.t;hint:modelData.k==="back"?"Backspace":modelData.t;tone:modelData.t==="="?"green":modelData.t==="×"?"gold":"cream";labelSize:modelData.t==="="?72:52;ink:modelData.t==="="?"#fcffdf":"#0a3220";bold:modelData.t==="=";onClicked:garden.calculate(modelData.k)
    Canvas{visible:modelData.k==="back";anchors.centerIn:parent;width:62;height:45;onPaint:{const c=getContext("2d");c.clearRect(0,0,width,height);c.strokeStyle="#0a3220";c.lineWidth=4;c.lineJoin="round";c.beginPath();c.moveTo(21,5);c.lineTo(57,5);c.lineTo(57,40);c.lineTo(21,40);c.lineTo(4,22.5);c.closePath();c.stroke();c.beginPath();c.moveTo(31,15);c.lineTo(46,30);c.moveTo(46,15);c.lineTo(31,30);c.stroke()}}
   }
  }
  Item {
   visible:host.historyOpen;x:856;y:205;width:434;height:913+garden.extraHeight
   Text{x:29;y:32;text:"History";font.family:"DejaVu Sans";font.pixelSize:41;font.bold:true;color:"#0b3420"}
   Hit{x:359;y:28;width:48;height:48;text:"‹";tone:"plain";labelSize:48;hint:"Hide history";onClicked:host.toggleHistory()}
   Item{x:47;y:117;width:34;height:35
    Rectangle{x:0;y:0;width:23;height:23;radius:12;color:"transparent";border.width:3;border.color:"#687a5b"}
    Rectangle{x:22;y:20;width:3;height:16;rotation:-42;color:"#687a5b"}
   }
   TextField{id:search;x:94;y:102;width:296;height:60;padding:0;placeholderText:"Search history";color:"#153d25";placeholderTextColor:"#687a5b";font.pixelSize:29;selectByMouse:true;background:Item{} Accessible.name:"Search history"}
   Text{visible:host.calcState.history.length===0;x:28;y:233;width:374;text:"No calculations yet";horizontalAlignment:Text.AlignHCenter;font.pixelSize:24;color:"#5d7650"}
   ListView {
    id:historyList;x:28;y:187;width:376;height:393+garden.extraHeight;clip:true;spacing:0
    model:host.calcState.history.map((e,i)=>({expression:e.expression,result:e.result,originalIndex:i})).filter(e=>!search.text||(e.expression+" "+e.result+" "+host.pretty(e.result)).toLowerCase().includes(search.text.toLowerCase()))
    ScrollBar.vertical:ScrollBar{policy:ScrollBar.AsNeeded}
    delegate:Item {
     id:entry;required property var modelData;required property int index;width:historyList.width;height:136
     property int originalIndex:modelData.originalIndex
     Rectangle{visible:host.scientific;width:parent.width;height:2;color:"#c4d3ac"}
     Rectangle{anchors.fill:parent;color:recall.containsMouse?"#33ffffff":"transparent";radius:8}
     Text{x:0;y:25;width:parent.width;text:garden.screenExpression(entry.modelData.expression);color:"#103921";font.pixelSize:30;elide:Text.ElideLeft}
     Text{x:0;y:72;width:parent.width;text:host.pretty(entry.modelData.result);color:"#0a331e";font.pixelSize:35;font.bold:true;horizontalAlignment:Text.AlignRight;elide:Text.ElideRight}
     MouseArea{id:recall;anchors.fill:parent;hoverEnabled:true;acceptedButtons:Qt.LeftButton|Qt.RightButton;onClicked:mouse=>{if(mouse.button===Qt.RightButton)entryMenu.popup();else garden.calculate("recall:"+entry.originalIndex)} onDoubleClicked:{host.act("reuse:"+entry.originalIndex);garden.editExpression()}}
     Menu{id:entryMenu;MenuItem{text:"Reuse expression";onTriggered:{host.act("reuse:"+entry.originalIndex);garden.editExpression()}}MenuItem{text:"Copy result";onTriggered:host.copyText(entry.modelData.result)}MenuItem{text:"Copy calculation";onTriggered:host.copyText(entry.modelData.expression+" = "+entry.modelData.result)}MenuItem{text:"Delete entry";onTriggered:host.act("delete:"+entry.originalIndex)}}
    }
   }
   // Keep bulk actions in the history context menu so the illustration stays clear.
   MouseArea{anchors.left:parent.left;anchors.right:parent.right;anchors.bottom:parent.bottom;height:230;acceptedButtons:Qt.RightButton;onClicked:historyMenu.popup()}
   Menu{id:historyMenu;MenuItem{text:"Copy all history";onTriggered:host.copyText(host.calcState.history.map(e=>e.expression+" = "+e.result).join("\n"))}MenuItem{text:"Clear history";onTriggered:garden.clearHistoryRequested()}}
  }
  Rectangle { visible:host.toast!=="";x:(parent.width-width)/2;y:parent.height-90;width:toastLabel.implicitWidth+40;height:50;radius:15;color:"#245333"
   Text{id:toastLabel;anchors.centerIn:parent;text:host.toast;color:"#fffced";font.pixelSize:23}
  }
  MouseArea{anchors.right:parent.right;anchors.bottom:parent.bottom;width:35;height:35;cursorShape:Qt.SizeFDiagCursor;onPressed:host.startSystemResize(Qt.RightEdge|Qt.BottomEdge)}
 }
}
