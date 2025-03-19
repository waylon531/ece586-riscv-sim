document.addEventListener("DOMContentLoaded", function() {
    var regs = ['ze','ra','sp','gp','tp','t0','t1','t2','s0','s1','a0','a1','a2','a3','a4','a5','a6','a7','s2','s3','s4','s5','s6','s7','s8','s9','s10','s11','t3','t4','t5','t6']
    var t = document.getElementById("registers");
    for(var r=0;r<4;r++) {
        t.insertRow();
        for(var c=0;c<8;c++) {
            t.rows[r].insertCell();
            t.rows[r].cells[c].innerHTML = "<span class='reg-label'>"+regs[(r*8 + c)] + "</span><br><span class='reg-value'>0x<span style='outline:none' tabindex=0 id='reg-"+(r*c + c) + "' class='reg-value-num' data-reg='"+(r*8 + c)+"'>00000000<span></span>";
        }
    } 
var activereg;
var setregto = ""
var digit = 0
const elements = document.querySelectorAll('.reg-value-num');
document.addEventListener('click', function(e) {
    if(e.target.classList.contains("reg-value-num")) return;
    try {
    activereg.classList.remove("activereg");
    activereg = null
    } catch {}
})
function toHex(d) {
    return  ("0000000"+(Number(d).toString(16))).slice(-8).toUpperCase()
}
function writeReg(a){
    fetch("http://localhost:9001/control/pokereg",{
        method: "POST",
        body: "reg="+(a-1)+"&val="+parseInt(setregto,16)
    }).then(response => response.json())
    .then(response => console.log(JSON.stringify(response)))
}
document.addEventListener('keydown',function(event) {
    if (activereg !== null) {
        key = keyboardMap[event.keyCode]
        if (["A","B","C","D","E","F","0","1","2","3","4","5","6","7","8","9"].includes(key)) {
            console.log(digit)
            console.log(setregto)
            if (digit==0) {
                setregto = "0000000"+key
                digit+=1
            } else if(digit<8){
                setregto=setregto.slice(1)+key
                digit+=1
            } else if(digit==8) {
                setregto = "0000000"+key
                digit=1
            }
            activereg.innerHTML = setregto;

        } else if(event.keyCode===13) {
            writeReg(activereg.dataset.reg)
            activereg.classList.add("setreg")
        }
    }
})
elements.forEach(element => {
  element.addEventListener('click', function(event) {
        if(element.dataset.reg==0) return
        try {
            activereg.classList.remove("activereg")
            activereg.classList.remove("setreg")
        } catch {}
        activereg = element;
        element.classList.add("activereg")
        setregto = "00000000";
        element.innerHTML="00000000"
/*  
     // Action when the element is clicked
     console.log(event.keyCode)
     if(event.keyCode===13){
        console.log("ENTER")
        return false;
     }*/
  });
});

})
function sendcmd(msg){
    setregto=0
    digit=0
    el=document.querySelector('.setreg')
    console.log(el)
    if(el!== null)
    el.classList.remove('setreg')
    function toHex(d) {
        return  ("0000000"+(Number(d).toString(16))).slice(-8).toUpperCase()
    }
    r = fetch("http://localhost:9001/control/"+msg,{
        method: "POST",
        body: "nothing interesting"
    }).then(response => response.json())
    .then(response => {
        document.getElementById("cycle").innerHTML=response.cycle
        l=document.createElement("li")
        l.textContent=toHex(response.pc) + ": " + response.cur_inst
        document.getElementById("inst").appendChild(l)
        r=document.querySelectorAll('.reg-value-num')
        r.forEach((element,i)=>{
            if(i==0) return
            console.log("reg "+i +": "+response.registers[i])
            var newval = toHex(response.registers[i-1])
            element.classList.remove("changedreg");
            if (newval!=element.innerHTML) {
                console.log("changed "+i)
                element.classList.add("changedreg");
            }
            element.innerHTML = toHex(response.registers[i-1])
        })
        //reg-value-num
    });
}
// names of known key codes (0-255)

var keyboardMap = [
  "", // [0]
  "", // [1]
  "", // [2]
  "CANCEL", // [3]
  "", // [4]
  "", // [5]
  "HELP", // [6]
  "", // [7]
  "BACK_SPACE", // [8]
  "TAB", // [9]
  "", // [10]
  "", // [11]
  "CLEAR", // [12]
  "ENTER", // [13]
  "ENTER_SPECIAL", // [14]
  "", // [15]
  "SHIFT", // [16]
  "CONTROL", // [17]
  "ALT", // [18]
  "PAUSE", // [19]
  "CAPS_LOCK", // [20]
  "KANA", // [21]
  "EISU", // [22]
  "JUNJA", // [23]
  "FINAL", // [24]
  "HANJA", // [25]
  "", // [26]
  "ESCAPE", // [27]
  "CONVERT", // [28]
  "NONCONVERT", // [29]
  "ACCEPT", // [30]
  "MODECHANGE", // [31]
  "SPACE", // [32]
  "PAGE_UP", // [33]
  "PAGE_DOWN", // [34]
  "END", // [35]
  "HOME", // [36]
  "LEFT", // [37]
  "UP", // [38]
  "RIGHT", // [39]
  "DOWN", // [40]
  "SELECT", // [41]
  "PRINT", // [42]
  "EXECUTE", // [43]
  "PRINTSCREEN", // [44]
  "INSERT", // [45]
  "DELETE", // [46]
  "", // [47]
  "0", // [48]
  "1", // [49]
  "2", // [50]
  "3", // [51]
  "4", // [52]
  "5", // [53]
  "6", // [54]
  "7", // [55]
  "8", // [56]
  "9", // [57]
  "COLON", // [58]
  "SEMICOLON", // [59]
  "LESS_THAN", // [60]
  "EQUALS", // [61]
  "GREATER_THAN", // [62]
  "QUESTION_MARK", // [63]
  "AT", // [64]
  "A", // [65]
  "B", // [66]
  "C", // [67]
  "D", // [68]
  "E", // [69]
  "F", // [70]
  "G", // [71]
  "H", // [72]
  "I", // [73]
  "J", // [74]
  "K", // [75]
  "L", // [76]
  "M", // [77]
  "N", // [78]
  "O", // [79]
  "P", // [80]
  "Q", // [81]
  "R", // [82]
  "S", // [83]
  "T", // [84]
  "U", // [85]
  "V", // [86]
  "W", // [87]
  "X", // [88]
  "Y", // [89]
  "Z", // [90]
  "OS_KEY", // [91] Windows Key (Windows) or Command Key (Mac)
  "", // [92]
  "CONTEXT_MENU", // [93]
  "", // [94]
  "SLEEP", // [95]
  "NUMPAD0", // [96]
  "NUMPAD1", // [97]
  "NUMPAD2", // [98]
  "NUMPAD3", // [99]
  "NUMPAD4", // [100]
  "NUMPAD5", // [101]
  "NUMPAD6", // [102]
  "NUMPAD7", // [103]
  "NUMPAD8", // [104]
  "NUMPAD9", // [105]
  "MULTIPLY", // [106]
  "ADD", // [107]
  "SEPARATOR", // [108]
  "SUBTRACT", // [109]
  "DECIMAL", // [110]
  "DIVIDE", // [111]
  "F1", // [112]
  "F2", // [113]
  "F3", // [114]
  "F4", // [115]
  "F5", // [116]
  "F6", // [117]
  "F7", // [118]
  "F8", // [119]
  "F9", // [120]
  "F10", // [121]
  "F11", // [122]
  "F12", // [123]
  "F13", // [124]
  "F14", // [125]
  "F15", // [126]
  "F16", // [127]
  "F17", // [128]
  "F18", // [129]
  "F19", // [130]
  "F20", // [131]
  "F21", // [132]
  "F22", // [133]
  "F23", // [134]
  "F24", // [135]
  "", // [136]
  "", // [137]
  "", // [138]
  "", // [139]
  "", // [140]
  "", // [141]
  "", // [142]
  "", // [143]
  "NUM_LOCK", // [144]
  "SCROLL_LOCK", // [145]
  "WIN_OEM_FJ_JISHO", // [146]
  "WIN_OEM_FJ_MASSHOU", // [147]
  "WIN_OEM_FJ_TOUROKU", // [148]
  "WIN_OEM_FJ_LOYA", // [149]
  "WIN_OEM_FJ_ROYA", // [150]
  "", // [151]
  "", // [152]
  "", // [153]
  "", // [154]
  "", // [155]
  "", // [156]
  "", // [157]
  "", // [158]
  "", // [159]
  "CIRCUMFLEX", // [160]
  "EXCLAMATION", // [161]
  "DOUBLE_QUOTE", // [162]
  "HASH", // [163]
  "DOLLAR", // [164]
  "PERCENT", // [165]
  "AMPERSAND", // [166]
  "UNDERSCORE", // [167]
  "OPEN_PAREN", // [168]
  "CLOSE_PAREN", // [169]
  "ASTERISK", // [170]
  "PLUS", // [171]
  "PIPE", // [172]
  "HYPHEN_MINUS", // [173]
  "OPEN_CURLY_BRACKET", // [174]
  "CLOSE_CURLY_BRACKET", // [175]
  "TILDE", // [176]
  "", // [177]
  "", // [178]
  "", // [179]
  "", // [180]
  "VOLUME_MUTE", // [181]
  "VOLUME_DOWN", // [182]
  "VOLUME_UP", // [183]
  "", // [184]
  "", // [185]
  "SEMICOLON", // [186]
  "EQUALS", // [187]
  "COMMA", // [188]
  "MINUS", // [189]
  "PERIOD", // [190]
  "SLASH", // [191]
  "BACK_QUOTE", // [192]
  "", // [193]
  "", // [194]
  "", // [195]
  "", // [196]
  "", // [197]
  "", // [198]
  "", // [199]
  "", // [200]
  "", // [201]
  "", // [202]
  "", // [203]
  "", // [204]
  "", // [205]
  "", // [206]
  "", // [207]
  "", // [208]
  "", // [209]
  "", // [210]
  "", // [211]
  "", // [212]
  "", // [213]
  "", // [214]
  "", // [215]
  "", // [216]
  "", // [217]
  "", // [218]
  "OPEN_BRACKET", // [219]
  "BACK_SLASH", // [220]
  "CLOSE_BRACKET", // [221]
  "QUOTE", // [222]
  "", // [223]
  "META", // [224]
  "ALTGR", // [225]
  "", // [226]
  "WIN_ICO_HELP", // [227]
  "WIN_ICO_00", // [228]
  "", // [229]
  "WIN_ICO_CLEAR", // [230]
  "", // [231]
  "", // [232]
  "WIN_OEM_RESET", // [233]
  "WIN_OEM_JUMP", // [234]
  "WIN_OEM_PA1", // [235]
  "WIN_OEM_PA2", // [236]
  "WIN_OEM_PA3", // [237]
  "WIN_OEM_WSCTRL", // [238]
  "WIN_OEM_CUSEL", // [239]
  "WIN_OEM_ATTN", // [240]
  "WIN_OEM_FINISH", // [241]
  "WIN_OEM_COPY", // [242]
  "WIN_OEM_AUTO", // [243]
  "WIN_OEM_ENLW", // [244]
  "WIN_OEM_BACKTAB", // [245]
  "ATTN", // [246]
  "CRSEL", // [247]
  "EXSEL", // [248]
  "EREOF", // [249]
  "PLAY", // [250]
  "ZOOM", // [251]
  "", // [252]
  "PA1", // [253]
  "WIN_OEM_CLEAR", // [254]
  "" // [255]
];