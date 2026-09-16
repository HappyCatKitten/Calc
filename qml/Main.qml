import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts
import QtCore

ApplicationWindow {
 id: win
 width: catacalc ? (historyOpen?933:629) : (historyOpen?660:430); height: catacalc ? (scientific?973:826) : (scientific?744:584)
 minimumWidth: catacalc ? (historyOpen?733:495) : (historyOpen?660:430); minimumHeight: catacalc ? (scientific?765:649) : (scientific?710:550)
 visible: true; title: "Brainfuck Calculator"; color: (win.catacalc?"#fcf9ed":"#202121")
 flags: Qt.Window | Qt.FramelessWindowHint | (prefs.onTop ? Qt.WindowStaysOnTopHint : 0)
 palette.window: (win.catacalc?"#edf1dc":"#252626"); palette.windowText: (win.catacalc?"#153d25":"#eeeeee"); palette.base: (win.catacalc?"#fffbed":"#303131"); palette.text: (win.catacalc?"#153d25":"#eeeeee"); palette.button: (win.catacalc?"#f6f5e5":"#363737"); palette.buttonText: (win.catacalc?"#153d25":"#eeeeee"); palette.highlight: (win.catacalc?"#92c83e":"#ff9e0b"); palette.highlightedText: (win.catacalc?"#14331b":"#141414")
 property bool catacalc: prefs.catacalc
 property bool degrees: prefs.degrees
 property var activeEditor: catacalc ? garden.expressionEditor : editor
 function focusCalculator(){if(catacalc)garden.focusCalculator();else main.forceActiveFocus()}
 function copyText(text){calculator.copy(text);notify("Copied")}
 function toggleAngle(){prefs.degrees=!prefs.degrees;act(prefs.degrees?"deg":"rad")}
 function setCatacalc(enabled){prefs.catacalc=enabled;if(visibility!==Window.Maximized){width=enabled?(historyOpen?933:629):(historyOpen?660:430);height=enabled?(scientific?973:826):(scientific?744:584)}focusCalculator()}
 property bool historyOpen: prefs.historyOpen
 property bool scientific: prefs.scientific
 property var calcState: ({expression:"",result:"0",history:[],memory:null,error:false,notice:""})
 property string toast: ""
 property string historyQuery: ""
 Settings { id: prefs; property bool catacalc: false; property bool historyOpen: true; property bool scientific: false; property bool grouping: true; property bool degrees: true; property bool onTop: false }
 function pretty(raw) {
  const s=String(raw); if (!/^-?\d+(\.\d+)?(e[+-]?\d+)?$/i.test(s)) return s
  const exponent=s.split(/e/i); const parts=exponent[0].split(".")
  const separator=","
  if(prefs.grouping) parts[0]=parts[0].replace(/\B(?=(\d{3})+(?!\d))/g,separator)
  return parts[0]+(parts.length>1 ? "."+parts[1] : "")+(exponent.length>1 ? "e"+exponent[1] : "")
 }
 function normalize(s) { s=s.replace(/[\s\u00a0\u202f]/g,"");return s.replace(/,/g,"") }
 function expressionText(raw) { return raw.replace(/\d+(?:\.\d+)?(?:e[+-]?\d+)?/gi, s => pretty(s)).replace(/([+−×÷])/g," $1 ") }
 function notify(message){toast=message; toastTimer.restart()}
 function act(key) { calcState=calculator.action(key); if(calcState.notice) notify(calcState.notice) }
 function copyResult(){calculator.copy(calcState.result);notify("Result copied")}
 function pasteExpression(){act("edit:"+normalize(calculator.paste()));win.focusCalculator()}
 function toggleHistory(){const step=catacalc?304:230;const nextWidth=width+(historyOpen ? -step : step);historyOpen=!historyOpen;prefs.historyOpen=historyOpen;if(visibility!==Window.Maximized)width=nextWidth;win.focusCalculator()}
 function setScientific(enabled){if(scientific===enabled)return;const step=catacalc?147:160;const nextHeight=height+(enabled ? step : -step);scientific=enabled;prefs.scientific=enabled;if(visibility!==Window.Maximized)height=nextHeight;if(!settings.opened)win.focusCalculator()}
 function toggleScientific(){setScientific(!scientific)}
 Component.onCompleted:{act(prefs.degrees ? "deg" : "rad");win.focusCalculator()}
 Timer{id:toastTimer;interval:2400;onTriggered:toast=""}
 Shortcut{sequence:"Ctrl+H";onActivated:toggleHistory()}
 Shortcut{sequence:"Ctrl+Shift+S";onActivated:toggleScientific()}
 Shortcut{sequence:"Ctrl+C";enabled:!activeEditor.activeFocus;onActivated:copyResult()}
 Shortcut{sequence:"Ctrl+V";enabled:!activeEditor.activeFocus;onActivated:pasteExpression()}
 Shortcut{sequence:"Ctrl+Z";enabled:!activeEditor.activeFocus;onActivated:act("undo")}
 Shortcut{sequence:"Ctrl+Shift+Z";enabled:!activeEditor.activeFocus;onActivated:act("redo")}
 Shortcut{sequence:"Ctrl+L";onActivated:{activeEditor.forceActiveFocus();activeEditor.selectAll()}}
 Shortcut{sequence:"Escape";onActivated:{settings.close();act("C");win.focusCalculator()}}
 component Key: Button {
  id: key
  property bool accent:false
  property string hint: text
  property int size: 22
  implicitHeight:38;implicitWidth:44;hoverEnabled:true
  background:Rectangle{radius:6;color:key.down ? (key.accent?(win.catacalc?"#71a72c":"#db8500"):(win.catacalc?"#c7d5a8":"#505252")) : key.hovered ? (key.accent?(win.catacalc?"#b1dc60":"#ffb333"):(win.catacalc?"#e9efce":"#404242")) : key.accent?(win.catacalc?"#92c83e":"#ff9e0b"):(win.catacalc?"#fffbed":"#303131");border.width:key.visualFocus?1:0;border.color:(win.catacalc?"#b1dc60":"#ffb333");Behavior on color{ColorAnimation{duration:70}}}
  contentItem:Text{text:key.text;color:key.enabled ? (win.catacalc?"#153d25":"#fafafa") : (win.catacalc?"#73816b":"#707474");font.pixelSize:key.size;horizontalAlignment:Text.AlignHCenter;verticalAlignment:Text.AlignVCenter}
  ToolTip.visible:hovered;ToolTip.delay:700;ToolTip.text:hint
  Accessible.name:hint
 }
 component SmallButton: Key {size:19;implicitHeight:34;implicitWidth:40;background:Rectangle{radius:5;color:parent.down?(win.catacalc?"#c7d5a8":"#505252"):parent.hovered?(win.catacalc?"#dbe5c3":"#424444"):parent.accent?(win.catacalc?"#92c83e":"#ff9e0b"):"transparent"}}
 Item {
  id:main;visible:!catacalc;anchors.fill:parent;focus:!catacalc
  Keys.onPressed:event=>{
   if(event.modifiers & (Qt.ControlModifier|Qt.AltModifier|Qt.MetaModifier))return
   if(event.key===Qt.Key_Return||event.key===Qt.Key_Enter||event.key===Qt.Key_Equal){act("=");event.accepted=true}
   else if(event.key===Qt.Key_Backspace){act("back");event.accepted=true}
   else if(event.key===Qt.Key_Delete){act("CE");event.accepted=true}
   else if(event.text&&event.text.length===1&&"0123456789.,+-*/()%^!".includes(event.text)){act(({"-":"−","*":"×","/":"÷",",":"."})[event.text]||event.text);event.accepted=true}
  }
  Rectangle {
   id:titlebar;width:parent.width;height:44
   gradient:Gradient{GradientStop{position:0;color:(win.catacalc?"#fffbed":"#303131")}GradientStop{position:1;color:(win.catacalc?"#d2dfbc":"#292a2a")}}
   MouseArea{anchors.fill:parent;onPressed:win.startSystemMove();onDoubleClicked:win.visibility===Window.Maximized?win.showNormal():win.showMaximized()}
   SmallButton{x:9;y:5;text:"☰";hint:"Options and shortcuts";onClicked:settings.open()}
   Text{x:58;anchors.verticalCenter:parent.verticalCenter;text:catacalc ? "Catacalc" : scientific?"Scientific":"Standard";font.pixelSize:12;color:(win.catacalc?"#234c2d":"#969b9b")}
   Row{anchors.right:parent.right;anchors.rightMargin:7;anchors.verticalCenter:parent.verticalCenter;spacing:2
    SmallButton{text:"◷";hint:"History · Ctrl+H";accent:historyOpen;onClicked:toggleHistory()}
    SmallButton{text:"−";hint:"Minimize";onClicked:win.showMinimized()}
    SmallButton{text:"□";hint:"Maximize / restore";onClicked:win.visibility===Window.Maximized?win.showNormal():win.showMaximized()}
    SmallButton{text:"×";hint:"Close";onClicked:win.close()}
   }
  }
  Image {
   id:themeBanner; anchors.top:titlebar.bottom; width:parent.width; height:catacalc?132:0
   visible:catacalc; source:catacalc?"assets/catacalc-banner.png":""
   fillMode:Image.PreserveAspectCrop; verticalAlignment:Image.AlignVCenter; clip:true
  }
  Item{
   id:calcArea;anchors.top:themeBanner.bottom;anchors.bottom:parent.bottom;anchors.left:parent.left;width:parent.width-(historyOpen?230:0)
   Item{
    id:display;anchors.top:parent.top;width:parent.width;height:137
    TextField{
     id:editor;x:20;y:15;width:parent.width-40;height:38
     text:activeFocus ? calcState.expression : expressionText(calcState.expression)
     color:(win.catacalc?"#2b5034":"#d4d8d8");font.pixelSize:21;selectByMouse:true;placeholderText:"Enter a calculation";placeholderTextColor:(win.catacalc?"#658064":"#737979")
     background:Rectangle{color:editor.activeFocus?(win.catacalc?"#fffbed":"#2a2c2c"):"transparent";radius:4;border.width:editor.activeFocus?1:0;border.color:(win.catacalc?"#8aa879":"#585b5b")}
     onTextEdited:act("edit:"+normalize(text))
     Keys.onReturnPressed:event=>{event.accepted=true;act("=");win.focusCalculator()}
     Keys.onEnterPressed:event=>{event.accepted=true;act("=");win.focusCalculator()}
     ToolTip.visible:hovered;ToolTip.text:"Edit an expression · Ctrl+L";ToolTip.delay:900
     Accessible.name:"Expression"
    }
    Text{x:23;y:61;text:calcState.memory!==null?"M":"";color:(win.catacalc?"#527c24":"#ffac29");font.pixelSize:12}
    Text{
     x:23;y:60;width:parent.width-46;height:65
     text:pretty(calcState.result);color:calcState.error?(win.catacalc?"#a33722":"#ffba7a"):(win.catacalc?"#153d25":"#fafafa");font.family:"DejaVu Sans";font.weight:Font.DemiBold;font.pixelSize:calcState.error?22:56;minimumPixelSize:18;fontSizeMode:Text.Fit;horizontalAlignment:Text.AlignRight;verticalAlignment:Text.AlignVCenter;maximumLineCount:1
     MouseArea{anchors.fill:parent;acceptedButtons:Qt.RightButton;onClicked:resultMenu.popup()}
    }
   }
   RowLayout{
    id:memoryRow;anchors.top:display.bottom;anchors.left:parent.left;anchors.right:parent.right;anchors.margins:13;height:29;spacing:4
    Repeater{model:["MC","MR","MS","M+","M-"];delegate:SmallButton{required property string modelData;text:modelData;size:12;Layout.fillWidth:true;Layout.preferredHeight:29;enabled:!(modelData==="MC"||modelData==="MR")||calcState.memory!==null;hint:({"MC":"Clear memory","MR":"Recall memory","MS":"Store result","M+":"Add to memory","M-":"Subtract from memory"})[modelData];onClicked:{act(modelData);win.focusCalculator()}}}
    SmallButton{text:"ƒx";hint:"Scientific keys · Ctrl+Shift+S";accent:scientific;size:17;Layout.preferredHeight:29;onClicked:toggleScientific()}
   }
   GridLayout{
    id:science;visible:scientific;anchors.top:memoryRow.bottom;anchors.topMargin:7;anchors.left:parent.left;anchors.right:parent.right;anchors.leftMargin:13;anchors.rightMargin:13;height:scientific?153:0;columns:5;rowSpacing:6;columnSpacing:6
    Repeater{model:[{t:prefs.degrees?"DEG":"RAD",k:"angle",h:"Switch degrees / radians"},{t:"(",k:"("},{t:")",k:")"},{t:"π",k:"pi"},{t:"e",k:"e"},{t:"sin",k:"sin"},{t:"cos",k:"cos"},{t:"tan",k:"tan"},{t:"ln",k:"ln"},{t:"log",k:"log"},{t:"xʸ",k:"^"},{t:"x!",k:"!"},{t:"|x|",k:"abs"},{t:"↶",k:"undo",h:"Undo · Ctrl+Z"},{t:"↷",k:"redo",h:"Redo · Ctrl+Shift+Z"}]
     delegate:Key{required property var modelData;text:modelData.t;hint:modelData.h||modelData.t;size:16;Layout.fillWidth:true;Layout.fillHeight:true;onClicked:{if(modelData.k==="angle"){prefs.degrees=!prefs.degrees;act(prefs.degrees?"deg":"rad")}else act(modelData.k);win.focusCalculator()}}
    }
   }
   RowLayout{
    id:extras;anchors.top:scientific?science.bottom:memoryRow.bottom;anchors.topMargin:8;anchors.left:parent.left;anchors.right:parent.right;anchors.leftMargin:13;anchors.rightMargin:13;height:37;spacing:7
    Repeater{model:[{t:"%",k:"%",h:"Percent · 200 + 10% = 220"},{t:"±",k:"±",h:"Change sign"},{t:"√",k:"sqrt",h:"Square root"},{t:"x²",k:"square",h:"Square"},{t:"1/x",k:"reciprocal",h:"Reciprocal"},{t:"CE",k:"CE",h:"Clear current entry · Delete"}];delegate:Key{required property var modelData;text:modelData.t;hint:modelData.h;size:16;Layout.fillWidth:true;Layout.fillHeight:true;onClicked:{act(modelData.k);win.focusCalculator()}}}
   }
   GridLayout{
    id:keys;anchors.top:extras.bottom;anchors.topMargin:8;anchors.bottom:parent.bottom;anchors.left:parent.left;anchors.right:parent.right;anchors.leftMargin:13;anchors.rightMargin:13;anchors.bottomMargin:16;columns:4;rowSpacing:7;columnSpacing:7;uniformCellWidths:true;uniformCellHeights:true
    Repeater{
     model:[{t:"C",r:0,c:0,h:"Clear all · Escape"},{t:"⌫",k:"back",r:0,c:1,h:"Backspace"},{t:"÷",r:0,c:2},{t:"×",r:0,c:3,a:true},{t:"7",r:1,c:0},{t:"8",r:1,c:1},{t:"9",r:1,c:2},{t:"−",r:1,c:3},{t:"4",r:2,c:0},{t:"5",r:2,c:1},{t:"6",r:2,c:2},{t:"+",r:2,c:3},{t:"1",r:3,c:0},{t:"2",r:3,c:1},{t:"3",r:3,c:2},{t:"=",r:3,c:3,rs:2,a:true},{t:"0",r:4,c:0,cs:2},{t:".",k:".",r:4,c:2}]
     delegate:Key{required property var modelData;text:modelData.t;hint:modelData.h||modelData.t;accent:modelData.a||false;size:modelData.t==="="?35:26;Layout.row:modelData.r;Layout.column:modelData.c;Layout.rowSpan:modelData.rs||1;Layout.columnSpan:modelData.cs||1;Layout.fillWidth:true;Layout.fillHeight:true;Layout.preferredWidth:85;Layout.preferredHeight:56;onClicked:{act(modelData.k||modelData.t);win.focusCalculator()}}
    }
   }
  }
  Rectangle{
   id:panel;visible:historyOpen;width:230;anchors.right:parent.right;anchors.top:themeBanner.bottom;anchors.bottom:parent.bottom;color:(win.catacalc?"#edf1dc":"#252626")
   Rectangle{width:1;height:parent.height;color:(win.catacalc?"#c7d6af":"#424343")}
   Text{x:21;y:23;text:"History";color:(win.catacalc?"#173f27":"#f4f4f4");font.pixelSize:17}
   SmallButton{anchors.right:parent.right;anchors.rightMargin:8;y:12;text:"‹";hint:"Hide history";onClicked:toggleHistory()}
   TextField{id:search;x:18;y:59;width:parent.width-36;height:32;placeholderText:"Search history";placeholderTextColor:(win.catacalc?"#627958":"#8b9292");font.pixelSize:12;selectByMouse:true;onTextChanged:historyQuery=text.toLowerCase();background:Rectangle{color:(win.catacalc?"#fffbed":"#303131");radius:4} Accessible.name:"Search history"}
   Text{visible:calcState.history.length===0;anchors.centerIn:parent;text:"No calculations yet";color:(win.catacalc?"#627958":"#888b8b");font.pixelSize:14}
   ListView{
    anchors.top:search.bottom;anchors.topMargin:12;anchors.left:parent.left;anchors.right:parent.right;anchors.bottom:historyActions.top;anchors.leftMargin:18;anchors.rightMargin:18;clip:true;model:calcState.history;spacing:0
    ScrollBar.vertical:ScrollBar{policy:ScrollBar.AsNeeded}
    delegate:Item{
     id:entry;required property var modelData;required property int index
     property bool matches:!historyQuery||(modelData.expression+" "+modelData.result+" "+pretty(modelData.result)).toLowerCase().includes(historyQuery)
     visible:matches;width:ListView.view.width;height:matches?84:0
     Rectangle{width:parent.width;height:1;color:(win.catacalc?"#c7d6af":"#414242")}
     Rectangle{anchors.fill:parent;anchors.topMargin:1;color:recall.containsMouse?(win.catacalc?"#fffbed":"#303131"):"transparent"}
     Text{x:0;y:18;width:parent.width;text:expressionText(modelData.expression)+" =";color:(win.catacalc?"#4f6949":"#b7baba");font.pixelSize:14;elide:Text.ElideLeft}
     Text{anchors.right:parent.right;y:44;width:parent.width;text:pretty(modelData.result);color:(win.catacalc?"#153d25":"#eeeeee");font.pixelSize:19;horizontalAlignment:Text.AlignRight;elide:Text.ElideRight}
     MouseArea{id:recall;anchors.fill:parent;hoverEnabled:true;acceptedButtons:Qt.LeftButton|Qt.RightButton;onClicked:mouse=>{if(mouse.button===Qt.RightButton)entryMenu.popup();else {act("recall:"+index);win.focusCalculator()}} onDoubleClicked:{act("reuse:"+index);editor.forceActiveFocus()}}
     Menu{id:entryMenu;MenuItem{text:"Reuse expression";onTriggered:act("reuse:"+entry.index)}MenuItem{text:"Copy result";onTriggered:{calculator.copy(entry.modelData.result);notify("Result copied")}}MenuItem{text:"Copy calculation";onTriggered:calculator.copy(entry.modelData.expression+" = "+entry.modelData.result)}MenuSeparator{}MenuItem{text:"Delete entry";onTriggered:act("delete:"+entry.index)}}
    }
   }
   RowLayout{id:historyActions;anchors.bottom:parent.bottom;anchors.bottomMargin:13;anchors.left:parent.left;anchors.right:parent.right;anchors.leftMargin:16;anchors.rightMargin:16;height:30
    SmallButton{text:"Copy all";size:12;enabled:calcState.history.length>0;Layout.fillWidth:true;hint:"Copy history as text";onClicked:{calculator.copy(calcState.history.map(e=>e.expression+" = "+e.result).join("\n"));notify("History copied")}}
    SmallButton{text:"Clear";size:12;enabled:calcState.history.length>0;Layout.fillWidth:true;hint:"Clear saved history";onClicked:clearDialog.open()}
   }
  }
  Rectangle{visible:toast!=="";z:10;anchors.horizontalCenter:parent.horizontalCenter;anchors.bottom:parent.bottom;anchors.bottomMargin:23;width:toastLabel.implicitWidth+28;height:36;radius:7;color:(win.catacalc?"#315b38":"#484b4b");Text{id:toastLabel;anchors.centerIn:parent;text:toast;color:"#ffffff";font.pixelSize:13}}
  MouseArea{anchors.right:parent.right;anchors.bottom:parent.bottom;width:10;height:10;cursorShape:Qt.SizeFDiagCursor;onPressed:win.startSystemResize(Qt.RightEdge|Qt.BottomEdge)}
 }
 Catacalc{id:garden;anchors.fill:parent;visible:catacalc;host:win;onOptionsRequested:settings.open();onResultMenuRequested:resultMenu.popup();onClearHistoryRequested:clearDialog.open()}
 Menu{id:resultMenu;MenuItem{text:"Copy result";onTriggered:copyResult()}MenuItem{text:"Copy formatted result";onTriggered:{calculator.copy(pretty(calcState.result));notify("Formatted result copied")}}MenuItem{text:"Paste calculation";onTriggered:pasteExpression()}}
 component OptionSwitch: Button {
  id:option
  property string description
  property bool active:false
  signal changed(bool enabled)
  implicitHeight:68;Layout.fillWidth:true;hoverEnabled:true
  Accessible.role:Accessible.CheckBox;Accessible.checked:active;Accessible.name:text
  onClicked:changed(!active)
  background:Rectangle{radius:7;color:option.hovered?(win.catacalc?"#e0eacb":"#3d4141"):(win.catacalc?"#f5f6e8":"#323636");border.width:option.visualFocus?1:0;border.color:(win.catacalc?"#6d9c38":"#ffad29")}
  contentItem:Item{
   Text{x:14;y:11;text:option.text;color:(win.catacalc?"#153d25":"#fafafa");font.pixelSize:16;font.weight:Font.Medium}
   Text{x:14;y:36;text:option.description;color:(win.catacalc?"#516648":"#c1c8c8");font.pixelSize:12}
   Rectangle{anchors.right:parent.right;anchors.rightMargin:13;anchors.verticalCenter:parent.verticalCenter;width:66;height:30;radius:15;color:option.active?(win.catacalc?"#92c83e":"#ff9e0b"):(win.catacalc?"#708765":"#555d5d")
    Text{anchors.left:parent.left;anchors.leftMargin:option.active?10:28;anchors.verticalCenter:parent.verticalCenter;text:option.active?"On":"Off";color:option.active?"#191b1b":"#ffffff";font.pixelSize:12;font.bold:true}
    Rectangle{x:option.active?40:4;y:4;width:22;height:22;radius:11;color:option.active?"#ffffff":"#dde2e2"}
   }
  }
 }
 Popup{
  id:settings;x:10;y:45;width:Math.min(390,win.width-20);height:Math.min(optionsColumn.implicitHeight+36,win.height-60);padding:18;modal:false;focus:true;closePolicy:Popup.CloseOnEscape|Popup.CloseOnPressOutside
  background:Rectangle{color:(win.catacalc?"#eaf0d9":"#272b2b");radius:10;border.color:(win.catacalc?"#9ab384":"#626969")}
  ScrollView{
   anchors.fill:parent;clip:true;contentWidth:availableWidth
   ScrollBar.horizontal.policy:ScrollBar.AlwaysOff
   ColumnLayout{
    id:optionsColumn;width:settings.availableWidth;spacing:10
    RowLayout{Layout.fillWidth:true;Layout.bottomMargin:5
     Text{text:"Options";color:win.catacalc?"#153d25":"#ffffff";font.pixelSize:22;font.weight:Font.DemiBold;Layout.fillWidth:true}
     SmallButton{text:"×";hint:"Close options";onClicked:settings.close()}
    }
    OptionSwitch{text:"Catacalc mode";description:"A little caterpillar, a greener calculator";active:prefs.catacalc;onChanged:enabled=>setCatacalc(enabled)}
    OptionSwitch{text:"Scientific mode";description:active?"Extra math keys are visible":"Simple calculator keypad";active:scientific;onChanged:enabled=>setScientific(enabled)}
    OptionSwitch{text:"Digit grouping";description:"Separate thousands for readability";active:prefs.grouping;onChanged:enabled=>prefs.grouping=enabled}
    OptionSwitch{text:"Always on top";description:"Keep the calculator above other apps";active:prefs.onTop;onChanged:enabled=>prefs.onTop=enabled}
    Key{text:"Keyboard shortcuts";size:14;Layout.fillWidth:true;Layout.preferredHeight:40;onClicked:{settings.close();shortcuts.open()}}
   }
  }
 }
 Dialog{
  id:shortcuts;anchors.centerIn:Overlay.overlay;width:Math.min(390,win.width-20);title:"Keyboard shortcuts";modal:true;standardButtons:Dialog.Close
  background:Rectangle{color:(win.catacalc?"#eaf0d9":"#292e2e");radius:10;border.color:(win.catacalc?"#9ab384":"#626969")}
  contentItem:ColumnLayout{spacing:14
   Repeater{model:[{key:"Enter",action:"Calculate / repeat"},{key:"Ctrl+L",action:"Edit expression"},{key:"Ctrl+C / V",action:"Copy / paste"},{key:"Ctrl+Z",action:"Undo"},{key:"Ctrl+Shift+Z",action:"Redo"},{key:"Ctrl+H",action:"Show / hide history"},{key:"Ctrl+Shift+S",action:"Scientific on / off"},{key:"Escape",action:"Clear calculation"},{key:"Delete",action:"Clear current entry"}]
    delegate:RowLayout{required property var modelData;Layout.fillWidth:true;Text{text:modelData.key;color:(win.catacalc?"#4e7a25":"#ffb641");font.pixelSize:14;Layout.preferredWidth:135}Text{text:modelData.action;color:(win.catacalc?"#24472d":"#f1f4f4");font.pixelSize:14}}
   }
  }
 }
 Dialog{id:clearDialog;anchors.centerIn:Overlay.overlay;title:"Clear all history?";modal:true;standardButtons:Dialog.Cancel|Dialog.Ok;onAccepted:act("clearHistory");Label{text:"This removes your saved calculations."}}
}
