document.addEventListener("DOMContentLoaded", function() {
    var regs = ['ze','ra','sp','gp','tp','t0','t1','t2','s0','s1','a0','a1','a2','a3','a4','a5','a6','a7','s2','s3','s4','s5','s6','s7','s8','s9','s10','s11','t3','t4','t5','t6']
    var t = document.getElementById("registers");
    for(var r=0;r<4;r++) {
        t.insertRow();
        for(var c=0;c<8;c++) {
            t.rows[r].insertCell();
            t.rows[r].cells[c].innerHTML = "<span class='reg-label'>"+regs[(r*8 + c)] + "</span><br><span id='reg-"+(r*c + c) + "' class='reg-value'>0x<span contenteditable>00<span></span>";
        }
    } 
})