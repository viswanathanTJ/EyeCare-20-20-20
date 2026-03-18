(function () {
  var C = 2 * Math.PI * 100;
  var breakDur = 20, breakRem = 20, bTimer = null, paused = false, total = 1200;
  var curInt = 20, curBrk = 20, curSnzDur = 5, curSnzMax = 3;
  var themeChoice = "system";
  var defaultAccent = "#64ffda", defaultPink = "#ffd700";
  var customAccent = localStorage.getItem("accent") || "";
  var customPink = localStorage.getItem("pink") || "";

  function applyCustomColors() {
    var r = document.documentElement.style;
    if (customAccent) {
      r.setProperty("--accent", customAccent);
      document.getElementById("accentPicker").value = customAccent;
    }
    if (customPink) {
      r.setProperty("--pink", customPink);
      document.getElementById("pinkPicker").value = customPink;
    }
  }
  // Apply on load
  if (customAccent || customPink) {
    document.addEventListener("DOMContentLoaded", applyCustomColors);
  }

  function pad(n) { return String(n).padStart(2, "0"); }

  // Click-to-edit: replaces a span with an input, Enter to save, Escape to cancel
  function isEditing(span) { return span && span.dataset.editing === "1"; }

  function makeEditable(spanId, opts) {
    var span = document.getElementById(spanId);
    if (!span) return;
    span.onclick = function () {
      if (span.dataset.editing === "1") return; // already editing
      span.dataset.editing = "1";
      var cur = opts.getValue();
      var input = document.createElement("input");
      input.type = "number";
      input.className = "cfg-val-input" + (opts.colorClass ? " " + opts.colorClass : "");
      input.value = cur;
      input.min = opts.min;
      input.max = opts.max;
      input.style.width = "60px";
      span.textContent = "";
      span.appendChild(input);
      input.focus();
      input.select();

      function close(val) {
        delete span.dataset.editing;
        span.textContent = opts.format(val);
      }

      input.onkeydown = function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          var v = parseInt(input.value, 10);
          if (isNaN(v) || v < opts.min || v > opts.max) {
            close(cur); // revert on invalid
          } else {
            opts.setValue(v);
            close(v);
          }
        }
        if (e.key === "Escape") {
          e.preventDefault();
          close(cur); // revert
        }
      };
      // On blur (click away), revert — don't save
      input.onblur = function () { close(cur); };
    };
  }

  function applyTheme(theme, systemTheme) {
    var isDark;
    if (theme === "system") {
      isDark = (systemTheme === "dark");
    } else {
      isDark = (theme === "dark");
    }
    if (isDark) {
      document.body.classList.remove("light");
    } else {
      document.body.classList.add("light");
    }
    document.querySelectorAll(".theme-btn").forEach(function (b) {
      b.classList.toggle("active", b.dataset.theme === theme);
    });
  }

  function pollState() {
    if (!window.__TAURI__) return;
    window.__TAURI__.core.invoke("get_state").then(function (s) {
      total = s.break_interval_secs;
      breakDur = s.break_duration_secs;
      curInt = Math.round(s.break_interval_secs / 60);
      curBrk = s.break_duration_secs;
      curSnzDur = Math.round(s.snooze_duration_secs / 60);
      curSnzMax = s.max_snoozes;

      document.getElementById("timerTime").textContent = pad(Math.floor(s.seconds_remaining / 60)) + ":" + pad(s.seconds_remaining % 60);
      document.getElementById("progress").style.strokeDashoffset = C * (1 - s.seconds_remaining / total);

      var b = document.getElementById("statusBadge");
      b.className = "badge";
      if (s.status === "Paused") {
        b.textContent = "Paused"; b.className += " badge-paused";
        document.getElementById("timerLabel").textContent = "paused";
        document.getElementById("pauseBtn").textContent = "Resume"; paused = true;
      } else if (s.status === "OnBreak") {
        b.textContent = "On Break"; b.className += " badge-break";
        document.getElementById("timerLabel").textContent = "break time";
      } else if (s.status === "Snoozed") {
        b.textContent = "Snoozed"; b.className += " badge-snoozed";
        var snzM = Math.floor(s.snooze_remaining / 60);
        var snzS = s.snooze_remaining % 60;
        document.getElementById("timerLabel").textContent = "snooze " + pad(snzM) + ":" + pad(snzS);
        // Update break overlay ring with snooze countdown
        if (document.getElementById("breakOverlay").classList.contains("show")) {
          document.getElementById("breakCountdown").textContent = pad(snzM) + ":" + pad(snzS);
          document.getElementById("breakProgress").style.strokeDashoffset = C * (1 - s.snooze_remaining / s.snooze_duration_secs);
          document.querySelector("#breakOverlay .ring-wrap .ring-center .ring-label").textContent = "snooze remaining";
        }
      } else {
        b.textContent = "Monitoring"; b.className += " badge-active";
        document.getElementById("timerLabel").textContent = "until next break";
        document.getElementById("pauseBtn").textContent = "Pause"; paused = false;
      }

      // Update settings panel values (skip if user is editing)
      if (!isEditing(document.getElementById("intVal")))
        document.getElementById("intVal").textContent = curInt + " min";
      if (!isEditing(document.getElementById("brkVal")))
        document.getElementById("brkVal").textContent = curBrk + " sec";

      // Stats panel
      document.getElementById("spCompleted").textContent = s.today_completed || 0;
      document.getElementById("spSkipped").textContent = s.today_skipped || 0;
      document.getElementById("spStreak").textContent = s.current_streak || 0;
      document.getElementById("spBest").textContent = s.longest_streak || 0;
      document.getElementById("spTotal").textContent = s.total_breaks || 0;

      // Toggles
      var autoTog = document.getElementById("autoStartToggle");
      autoTog.className = s.auto_start ? "toggle on" : "toggle";

      var timerTog = document.getElementById("timerMenuToggle");
      timerTog.className = s.show_timer_in_menu ? "toggle on" : "toggle";

      var soundTog = document.getElementById("soundToggle");
      soundTog.className = s.sound_enabled ? "toggle on" : "toggle";

      // Header sound icon
      var soundBtn = document.getElementById("soundBtn");
      var soundWaves = document.getElementById("soundWaves");
      if (s.sound_enabled) {
        soundBtn.classList.add("active");
        soundBtn.title = "Sound On";
        soundWaves.style.display = "";
      } else {
        soundBtn.classList.remove("active");
        soundBtn.title = "Sound Off";
        soundWaves.style.display = "none";
      }

      // Main panel config display (skip if editing)
      if (!isEditing(document.getElementById("intValMain")))
        document.getElementById("intValMain").textContent = curInt + " min";
      if (!isEditing(document.getElementById("brkValMain")))
        document.getElementById("brkValMain").textContent = curBrk + " sec";

      // Snooze settings display (skip if editing)
      if (!isEditing(document.getElementById("snzDurVal")))
        document.getElementById("snzDurVal").textContent = curSnzDur + " min";
      if (!isEditing(document.getElementById("snzMaxVal")))
        document.getElementById("snzMaxVal").textContent = curSnzMax;

      // Snooze button label + visibility
      document.getElementById("snoozeBtn").textContent = "Snooze " + curSnzDur + "m";
      if (s.snooze_count >= s.max_snoozes) {
        document.getElementById("snoozeBtn").style.display = "none";
        document.getElementById("snoozeWrap").style.display = "none";
      }

      // Reminder mode (settings panel + dashboard chips)
      document.querySelectorAll(".mode-row").forEach(function (r) {
        r.classList.toggle("active", r.dataset.mode === s.reminder_mode);
      });
      document.querySelectorAll(".chip").forEach(function (c) {
        c.className = c.dataset.mode === s.reminder_mode ? "chip on" : "chip";
      });

      // Theme
      applyTheme(themeChoice, s.theme);
      applyCustomColors();
    });
  }

  function showBreak(tip) {
    document.getElementById("breakTip").textContent = '"' + (tip || "Look 20 feet away.") + '"';
    breakRem = breakDur;
    updBreak();
    document.getElementById("breakOverlay").className = "overlay show";
    document.getElementById("breakControls").style.display = "flex";
    document.getElementById("startBreakBtn").style.display = "";
    document.getElementById("snoozeBtn").style.display = "";
    document.getElementById("snoozeBtn").textContent = "Snooze " + curSnzDur + "m";
    document.getElementById("snoozeWrap").style.display = "";
    document.getElementById("completeMsg").style.display = "none";
    document.getElementById("breakTitle").textContent = "Time for an Eye Break";
    document.getElementById("breakTitle").style.display = "";
    document.getElementById("breakTip").style.display = "";
    document.querySelector("#breakOverlay .ring-wrap").style.display = "";
  }

  function updBreak() {
    document.getElementById("breakCountdown").textContent = breakRem;
    document.getElementById("breakProgress").style.strokeDashoffset = C * (1 - breakRem / breakDur);
  }

  function hideBreak() {
    document.getElementById("breakOverlay").className = "overlay";
    if (celebrateTimer) { clearInterval(celebrateTimer); celebrateTimer = null; }
    document.getElementById("confettiBox").innerHTML = "";
  }

  var celebrateTimer = null;

  var confettiColors = ["#e74c3c", "#3498db", "#f1c40f", "#2ecc71", "#e67e22", "#9b59b6", "#1abc9c", "#ff6b9d"];

  function spawnConfetti() {
    var box = document.getElementById("confettiBox");
    for (var i = 0; i < 8; i++) {
      var isDot = Math.random() > 0.6;
      var el = document.createElement("div");
      el.className = isDot ? "confetti-dot" : "confetti-piece";
      el.style.left = Math.random() * 100 + "%";
      el.style.background = confettiColors[Math.floor(Math.random() * confettiColors.length)];
      el.style.animationDuration = (2.5 + Math.random() * 2) + "s";
      el.style.animationDelay = (Math.random() * 0.5) + "s";
      if (!isDot) {
        el.style.width = (6 + Math.random() * 10) + "px";
        el.style.height = (4 + Math.random() * 4) + "px";
        el.style.borderRadius = Math.random() > 0.5 ? "2px" : "50%";
      }
      box.appendChild(el);
      (function (e) { setTimeout(function () { e.remove(); }, 5000); })(el);
    }
  }

  // Initial burst — spawn many at once
  function burstConfetti() {
    for (var i = 0; i < 6; i++) {
      setTimeout(spawnConfetti, i * 100);
    }
  }

  function showCelebration() {
    // Initial big burst then continuous
    burstConfetti();
    var confettiInterval = setInterval(spawnConfetti, 400);

    // Progress bar countdown (5 seconds)
    var prog = document.getElementById("celebrateProgress");
    prog.style.transition = "none";
    prog.style.width = "100%";
    setTimeout(function () {
      prog.style.transition = "width 5s linear";
      prog.style.width = "0%";
    }, 50);

    celebrateTimer = setTimeout(function () {
      clearInterval(confettiInterval);
      hideBreak();
    }, 5000);

    // Hurray button — dismiss early
    document.getElementById("hurrayBtn").onclick = function () {
      clearInterval(confettiInterval);
      hideBreak();
    };
  }

  function toggleSettings() {
    var panel = document.getElementById("settingsPanel");
    var btn = document.getElementById("settingsBtn");
    var isOpen = panel.classList.contains("show");
    if (isOpen) {
      panel.classList.remove("show");
      btn.classList.remove("active");
    } else {
      panel.classList.add("show");
      btn.classList.add("active");
    }
  }

  function init() {
    if (!window.__TAURI__ || !window.__TAURI__.core) { setTimeout(init, 200); return; }
    var T = window.__TAURI__;

    T.core.invoke("get_tip").then(function (t) { document.getElementById("tipBox").textContent = '"' + t + '"'; });

    T.event.listen("break-due", function () {
      T.core.invoke("get_tip").then(function (t) { showBreak(t); }).catch(function () { showBreak(null); });
      // Hide snooze button if max snoozes already reached
      T.core.invoke("get_state").then(function (s) {
        if (s.snooze_count >= s.max_snoozes) {
          document.getElementById("snoozeBtn").style.display = "none";
          document.getElementById("snoozeWrap").style.display = "none";
        }
      });
    });

    T.event.listen("break-snoozed", function () {
      // Keep the break overlay visible — just stop break timer if running
      if (bTimer) clearInterval(bTimer);
    });

    // Settings toggle
    document.getElementById("settingsBtn").onclick = toggleSettings;
    document.getElementById("settingsBack").onclick = toggleSettings;

    // Stats toggle
    document.getElementById("statsBtn").onclick = function () {
      var panel = document.getElementById("statsPanel");
      var btn = document.getElementById("statsBtn");
      var isOpen = panel.classList.contains("show");
      if (isOpen) {
        panel.classList.remove("show"); btn.classList.remove("active");
      } else {
        panel.classList.add("show"); btn.classList.add("active");
      }
    };
    document.getElementById("statsBack").onclick = function () {
      document.getElementById("statsPanel").classList.remove("show");
      document.getElementById("statsBtn").classList.remove("active");
    };

    // Quit button
    document.getElementById("quitBtn").onclick = function () {
      T.core.invoke("quit_app");
    };

    // Reset stats with inline confirmation
    document.getElementById("resetStatsBtn").onclick = function () {
      document.getElementById("resetConfirm").style.display = "block";
      this.style.display = "none";
    };
    document.getElementById("resetYes").onclick = function () {
      T.core.invoke("reset_stats");
      document.getElementById("resetConfirm").style.display = "none";
      document.getElementById("resetStatsBtn").style.display = "";
    };
    document.getElementById("resetNo").onclick = function () {
      document.getElementById("resetConfirm").style.display = "none";
      document.getElementById("resetStatsBtn").style.display = "";
    };

    // Dashboard buttons
    document.getElementById("takeBreakBtn").onclick = function () {
      T.core.invoke("take_break_now").then(function () { return T.core.invoke("get_tip"); })
        .then(function (t) { showBreak(t); }).catch(function () { showBreak(null); });
    };

    document.getElementById("pauseBtn").onclick = function () {
      T.core.invoke(paused ? "resume_monitoring" : "pause_monitoring");
    };

    // Break controls
    document.getElementById("startBreakBtn").onclick = function () {
      document.getElementById("startBreakBtn").style.display = "none";
      breakRem = breakDur;
      bTimer = setInterval(function () {
        breakRem--; updBreak();
        if (breakRem <= 0) {
          clearInterval(bTimer);
          T.core.invoke("finish_break");
          // Play completion sound
          T.core.invoke("play_complete_sound");
          // Show celebration
          document.getElementById("breakControls").style.display = "none";
          document.getElementById("snoozeWrap").style.display = "none";
          document.getElementById("breakTitle").style.display = "none";
          document.getElementById("breakTip").style.display = "none";
          document.querySelector("#breakOverlay .ring-wrap").style.display = "none";
          document.getElementById("completeMsg").style.display = "block";
          T.core.invoke("get_state").then(function (s) {
            document.getElementById("celebrateCount").textContent = s.today_completed || 0;
          });
          showCelebration();
        }
      }, 1000);
    };

    document.getElementById("snoozeBtn").onclick = function () {
      if (bTimer) clearInterval(bTimer);
      T.core.invoke("snooze_break").then(function () {
        // Keep overlay visible — user can still click Start Break or Skip
        document.getElementById("breakTitle").textContent = "Snoozed";
      }).catch(function () {
        // Max snoozes reached — do nothing
      });
    };

    document.getElementById("skipBtn").onclick = function () {
      if (bTimer) clearInterval(bTimer);
      T.core.invoke("skip_break");
      hideBreak();
    };

    // Interval +/- (both settings panel and main panel)
    // Snap to next/prev multiple of step
    function snapUp(val, step, max) { return Math.min(max, Math.floor(val / step) * step + step); }
    function snapDown(val, step, min) { return Math.max(min, Math.ceil(val / step) * step - step); }

    function intUp() { curInt = snapUp(curInt, 5, 120); T.core.invoke("set_interval", { minutes: curInt }); }
    function intDown() { curInt = snapDown(curInt, 5, 5); T.core.invoke("set_interval", { minutes: curInt }); }
    function brkUp() { curBrk = snapUp(curBrk, 5, 60); T.core.invoke("set_break_duration", { seconds: curBrk }); }
    function brkDown() { curBrk = snapDown(curBrk, 5, 5); T.core.invoke("set_break_duration", { seconds: curBrk }); }

    document.getElementById("intUp").onclick = intUp;
    document.getElementById("intDown").onclick = intDown;
    document.getElementById("brkUp").onclick = brkUp;
    document.getElementById("brkDown").onclick = brkDown;
    document.getElementById("intUpMain").onclick = intUp;
    document.getElementById("intDownMain").onclick = intDown;
    document.getElementById("brkUpMain").onclick = brkUp;
    document.getElementById("brkDownMain").onclick = brkDown;

    // Snooze duration +/-
    document.getElementById("snzDurUp").onclick = function () {
      curSnzDur = Math.min(curInt - 1, curSnzDur + 1);
      T.core.invoke("set_snooze_duration", { minutes: curSnzDur });
    };
    document.getElementById("snzDurDown").onclick = function () {
      curSnzDur = Math.max(1, curSnzDur - 1);
      T.core.invoke("set_snooze_duration", { minutes: curSnzDur });
    };
    // Max snoozes +/-
    document.getElementById("snzMaxUp").onclick = function () {
      curSnzMax = Math.min(10, curSnzMax + 1);
      T.core.invoke("set_max_snoozes", { count: curSnzMax });
    };
    document.getElementById("snzMaxDown").onclick = function () {
      curSnzMax = Math.max(1, curSnzMax - 1);
      T.core.invoke("set_max_snoozes", { count: curSnzMax });
    };

    // Click-to-edit: Interval (settings + dashboard)
    var intEditOpts = {
      getValue: function () { return curInt; },
      setValue: function (v) { curInt = v; T.core.invoke("set_interval", { minutes: v }); },
      min: 5, max: 120,
      format: function (v) { return v + " min"; },
      colorClass: ""
    };
    makeEditable("intVal", intEditOpts);
    makeEditable("intValMain", intEditOpts);

    // Click-to-edit: Break duration (settings + dashboard)
    var brkEditOpts = {
      getValue: function () { return curBrk; },
      setValue: function (v) { curBrk = v; T.core.invoke("set_break_duration", { seconds: v }); },
      min: 5, max: 60,
      format: function (v) { return v + " sec"; },
      colorClass: "pink"
    };
    makeEditable("brkVal", brkEditOpts);
    makeEditable("brkValMain", brkEditOpts);

    // Click-to-edit: Snooze duration
    makeEditable("snzDurVal", {
      getValue: function () { return curSnzDur; },
      setValue: function (v) { curSnzDur = v; T.core.invoke("set_snooze_duration", { minutes: v }); },
      min: 1, max: 59,
      format: function (v) { return v + " min"; }
    });

    // Click-to-edit: Max snoozes
    makeEditable("snzMaxVal", {
      getValue: function () { return curSnzMax; },
      setValue: function (v) { curSnzMax = v; T.core.invoke("set_max_snoozes", { count: v }); },
      min: 1, max: 10,
      format: function (v) { return String(v); }
    });

    // Dashboard mode chips
    document.querySelectorAll(".chip").forEach(function (c) {
      c.onclick = function () { T.core.invoke("set_reminder_mode", { mode: c.dataset.mode }); };
    });

    // Settings: Timer in menu bar toggle
    document.getElementById("timerMenuToggle").onclick = function () {
      T.core.invoke("toggle_timer_in_menu");
    };

    // Settings: Reminder modes
    document.querySelectorAll(".mode-row").forEach(function (r) {
      r.onclick = function () { T.core.invoke("set_reminder_mode", { mode: r.dataset.mode }); };
    });

    // Settings: Auto-start toggle
    document.getElementById("autoStartToggle").onclick = function () {
      var isOn = this.classList.contains("on");
      T.core.invoke("set_auto_start", { enabled: !isOn });
    };

    document.getElementById("soundToggle").onclick = function () {
      T.core.invoke("toggle_sound");
    };

    // Header sound button
    document.getElementById("soundBtn").onclick = function () {
      T.core.invoke("toggle_sound");
    };

    // GitHub link
    document.getElementById("githubLink").onclick = function (e) {
      e.preventDefault();
      T.core.invoke("open_url", { url: "https://github.com/viswanathanTJ/EyeCare-20-20-20/" });
    };

    // Settings: Theme buttons
    document.querySelectorAll(".theme-btn").forEach(function (b) {
      b.onclick = function () {
        themeChoice = b.dataset.theme;
        T.core.invoke("get_state").then(function (s) {
          applyTheme(themeChoice, s.theme);
          applyCustomColors();
        });
      };
    });

    // Custom color pickers
    document.getElementById("accentPicker").oninput = function () {
      customAccent = this.value;
      document.documentElement.style.setProperty("--accent", customAccent);
      localStorage.setItem("accent", customAccent);
    };
    document.getElementById("pinkPicker").oninput = function () {
      customPink = this.value;
      document.documentElement.style.setProperty("--pink", customPink);
      localStorage.setItem("pink", customPink);
    };
    document.getElementById("resetColorsBtn").onclick = function () {
      customAccent = ""; customPink = "";
      localStorage.removeItem("accent");
      localStorage.removeItem("pink");
      document.documentElement.style.removeProperty("--accent");
      document.documentElement.style.removeProperty("--pink");
      document.getElementById("accentPicker").value = defaultAccent;
      document.getElementById("pinkPicker").value = defaultPink;
    };

    // Apply saved custom colors
    applyCustomColors();

    // Initial sync
    T.core.invoke("get_state").then(function (s) {
      curInt = Math.round(s.break_interval_secs / 60);
      curBrk = s.break_duration_secs;
      curSnzDur = Math.round(s.snooze_duration_secs / 60);
      curSnzMax = s.max_snoozes;
    });

    // Keyboard shortcuts
    document.addEventListener("keydown", function (e) {
      // Escape — close settings or stats panel
      if (e.key === "Escape" || e.key === "Backspace") {
        var settings = document.getElementById("settingsPanel");
        var stats = document.getElementById("statsPanel");
        if (settings.classList.contains("show")) {
          settings.classList.remove("show");
          document.getElementById("settingsBtn").classList.remove("active");
        } else if (stats.classList.contains("show")) {
          stats.classList.remove("show");
          document.getElementById("statsBtn").classList.remove("active");
        }
      }
      // Enter or Space — start break or dismiss celebration
      if (e.key === "Enter" || e.key === " ") {
        var overlay = document.getElementById("breakOverlay");
        if (!overlay.classList.contains("show")) return;
        e.preventDefault();
        var hurray = document.getElementById("hurrayBtn");
        var completeVisible = document.getElementById("completeMsg").style.display !== "none";
        if (completeVisible) {
          hurray.click();
        } else {
          var startBtn = document.getElementById("startBreakBtn");
          if (startBtn.style.display !== "none") {
            startBtn.click();
          }
        }
      }
    });

    pollState();
    setInterval(pollState, 1000);
  }

  init();
})();
