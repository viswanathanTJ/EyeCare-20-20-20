(function () {
  var C = 2 * Math.PI * 100;
  var breakDur = 20, breakRem = 20, bTimer = null, paused = false, total = 1200;
  var curInt = 20, curBrk = 20, curSnzDur = 5, curSnzMax = 3;
  var settingsDirty = 0; // timestamp of last user setting change
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

  function markDirty() { settingsDirty = Date.now(); }
  function isSettingsDirty() { return Date.now() - settingsDirty < 800; }

  // Debounce: delays fn until wait ms after last call
  var _debounceTimers = {};
  function debounce(key, fn, wait) {
    if (_debounceTimers[key]) clearTimeout(_debounceTimers[key]);
    _debounceTimers[key] = setTimeout(fn, wait);
  }

  // Immediately update all settings displays from local vars
  function refreshSettingsUI() {
    var el;
    el = document.getElementById("intVal");
    if (el && !isEditing(el)) el.textContent = curInt + " min";
    el = document.getElementById("intValMain");
    if (el && !isEditing(el)) el.textContent = curInt + " min";
    el = document.getElementById("brkVal");
    if (el && !isEditing(el)) el.textContent = curBrk + " sec";
    el = document.getElementById("brkValMain");
    if (el && !isEditing(el)) el.textContent = curBrk + " sec";
    el = document.getElementById("snzDurVal");
    if (el && !isEditing(el)) el.textContent = curSnzDur + " min";
    el = document.getElementById("snzMaxVal");
    if (el && !isEditing(el)) el.textContent = String(curSnzMax);
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

  // Cached DOM refs (populated on first pollState call)
  var $ = {};
  function cacheDOM() {
    if ($.timerTime) return;
    var ids = [
      "timerTime", "progress", "statusBadge", "timerLabel", "pauseBtn",
      "breakOverlay", "breakCountdown", "breakProgress",
      "intVal", "intValMain", "brkVal", "brkValMain",
      "snzDurVal", "snzMaxVal", "snoozeBtn", "snoozeWrap",
      "spCompleted", "spSkipped", "spStreak", "spBest", "spTotal",
      "autoStartToggle", "timerMenuToggle", "soundToggle",
      "soundBtn", "soundWaves"
    ];
    ids.forEach(function (id) { $[id] = document.getElementById(id); });
    $.snoozeRingLabel = document.querySelector("#breakOverlay .ring-wrap .ring-center .ring-label");
    $.modeRows = document.querySelectorAll(".mode-row");
    $.chips = document.querySelectorAll(".chip");
  }

  function pollState() {
    if (!window.__TAURI__) return;
    cacheDOM();
    window.__TAURI__.core.invoke("get_state").then(function (s) {
      total = s.break_interval_secs;
      breakDur = s.break_duration_secs;

      $.timerTime.textContent = pad(Math.floor(s.seconds_remaining / 60)) + ":" + pad(s.seconds_remaining % 60);
      $.progress.style.strokeDashoffset = C * (1 - s.seconds_remaining / total);

      $.statusBadge.className = "badge";
      if (s.status === "Paused") {
        $.statusBadge.textContent = "Paused"; $.statusBadge.className += " badge-paused";
        $.timerLabel.textContent = "paused";
        $.pauseBtn.textContent = "Resume"; paused = true;
      } else if (s.status === "OnBreak") {
        $.statusBadge.textContent = "On Break"; $.statusBadge.className += " badge-break";
        $.timerLabel.textContent = "break time";
      } else if (s.status === "Snoozed") {
        $.statusBadge.textContent = "Snoozed"; $.statusBadge.className += " badge-snoozed";
        var snzM = Math.floor(s.snooze_remaining / 60);
        var snzS = s.snooze_remaining % 60;
        $.timerLabel.textContent = "snooze " + pad(snzM) + ":" + pad(snzS);
        // Update break overlay ring with snooze countdown
        if ($.breakOverlay.classList.contains("show")) {
          $.breakCountdown.textContent = pad(snzM) + ":" + pad(snzS);
          $.breakProgress.style.strokeDashoffset = C * (1 - s.snooze_remaining / s.snooze_duration_secs);
          if ($.snoozeRingLabel) $.snoozeRingLabel.textContent = "snooze remaining";
        }
      } else {
        $.statusBadge.textContent = "Monitoring"; $.statusBadge.className += " badge-active";
        $.timerLabel.textContent = "until next break";
        $.pauseBtn.textContent = "Pause"; paused = false;
      }

      // Update settings panel values (skip if user is editing or just changed)
      if (!isSettingsDirty()) {
        curInt = Math.round(s.break_interval_secs / 60);
        curBrk = s.break_duration_secs;
        curSnzDur = Math.round(s.snooze_duration_secs / 60);
        curSnzMax = s.max_snoozes;
      }
      if (!isEditing($.intVal)) $.intVal.textContent = curInt + " min";
      if (!isEditing($.brkVal)) $.brkVal.textContent = curBrk + " sec";

      // Stats panel
      $.spCompleted.textContent = s.today_completed || 0;
      $.spSkipped.textContent = s.today_skipped || 0;
      $.spStreak.textContent = s.current_streak || 0;
      $.spBest.textContent = s.longest_streak || 0;
      $.spTotal.textContent = s.total_breaks || 0;

      // Toggles
      $.autoStartToggle.className = s.auto_start ? "toggle on" : "toggle";
      $.timerMenuToggle.className = s.show_timer_in_menu ? "toggle on" : "toggle";
      $.soundToggle.className = s.sound_enabled ? "toggle on" : "toggle";

      // Header sound icon
      if (s.sound_enabled) {
        $.soundBtn.classList.add("active");
        $.soundBtn.title = "Sound On";
        $.soundWaves.style.display = "";
      } else {
        $.soundBtn.classList.remove("active");
        $.soundBtn.title = "Sound Off";
        $.soundWaves.style.display = "none";
      }

      // Main panel config display (skip if editing)
      if (!isEditing($.intValMain)) $.intValMain.textContent = curInt + " min";
      if (!isEditing($.brkValMain)) $.brkValMain.textContent = curBrk + " sec";

      // Snooze settings display (skip if editing)
      if (!isEditing($.snzDurVal)) $.snzDurVal.textContent = curSnzDur + " min";
      if (!isEditing($.snzMaxVal)) $.snzMaxVal.textContent = curSnzMax;

      // Snooze button label + visibility
      $.snoozeBtn.textContent = "Snooze " + curSnzDur + "m";
      if (s.snooze_count >= s.max_snoozes) {
        $.snoozeBtn.style.display = "none";
        $.snoozeWrap.style.display = "none";
      }

      // Reminder mode (settings panel + dashboard chips)
      $.modeRows.forEach(function (r) {
        r.classList.toggle("active", r.dataset.mode === s.reminder_mode);
      });
      $.chips.forEach(function (c) {
        c.className = c.dataset.mode === s.reminder_mode ? "chip on" : "chip";
      });

      // Theme
      applyTheme(themeChoice, s.theme);
    });
  }

  function showBreak(tip, snoozeExhausted) {
    document.getElementById("breakTip").textContent = '"' + (tip || "Look 20 feet away.") + '"';
    breakRem = breakDur;
    updBreak();
    document.getElementById("breakOverlay").className = "overlay show";
    document.getElementById("breakControls").style.display = "flex";
    document.getElementById("startBreakBtn").style.display = "";
    document.getElementById("snoozeBtn").textContent = "Snooze " + curSnzDur + "m";
    if (snoozeExhausted) {
      document.getElementById("snoozeBtn").style.display = "none";
      document.getElementById("snoozeWrap").style.display = "none";
    } else {
      document.getElementById("snoozeBtn").style.display = "";
      document.getElementById("snoozeWrap").style.display = "";
    }
    document.getElementById("completeMsg").style.display = "none";
    document.getElementById("breakTitle").textContent = "Time for an Eye Break";
    document.getElementById("breakTitle").style.display = "";
    document.getElementById("breakTip").style.display = "";
    document.querySelector("#breakOverlay .ring-wrap").style.display = "";
  }

  function updBreak() {
    var m = Math.floor(breakRem / 60), s = breakRem % 60;
    document.getElementById("breakCountdown").textContent = m > 0 ? pad(m) + ":" + pad(s) : breakRem;
    document.getElementById("breakProgress").style.strokeDashoffset = C * (1 - breakRem / breakDur);
  }

  function hideBreak() {
    document.getElementById("breakOverlay").className = "overlay";
    if (celebrateTimer) { clearTimeout(celebrateTimer); celebrateTimer = null; }
    if (confettiInterval) { clearInterval(confettiInterval); confettiInterval = null; }
    document.getElementById("confettiBox").innerHTML = "";
  }

  var celebrateTimer = null;
  var confettiInterval = null;

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
    if (confettiInterval) clearInterval(confettiInterval);
    confettiInterval = setInterval(spawnConfetti, 400);

    // Progress bar countdown (5 seconds)
    var prog = document.getElementById("celebrateProgress");
    prog.style.transition = "none";
    prog.style.width = "100%";
    setTimeout(function () {
      prog.style.transition = "width 5s linear";
      prog.style.width = "0%";
    }, 50);

    celebrateTimer = setTimeout(function () {
      hideBreak();
    }, 5000);

    // Hurray button — dismiss early
    document.getElementById("hurrayBtn").onclick = function () {
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

  var _initRetries = 0;
  function init() {
    if (!window.__TAURI__ || !window.__TAURI__.core) {
      if (++_initRetries > 50) { console.error("Tauri API not available after 10s"); return; }
      setTimeout(init, 200); return;
    }
    var T = window.__TAURI__;

    T.core.invoke("get_tip").then(function (t) { document.getElementById("tipBox").textContent = '"' + t + '"'; });

    T.event.listen("break-due", function () {
      Promise.all([
        T.core.invoke("get_tip").catch(function () { return null; }),
        T.core.invoke("get_state")
      ]).then(function (results) {
        var tip = results[0], s = results[1];
        showBreak(tip, s.snooze_count >= s.max_snoozes);
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
      T.core.invoke("quit_app").catch(function (e) { console.error("quit_app:", e); });
    };

    // Reset stats with inline confirmation
    document.getElementById("resetStatsBtn").onclick = function () {
      document.getElementById("resetConfirm").style.display = "block";
      this.style.display = "none";
    };
    document.getElementById("resetYes").onclick = function () {
      T.core.invoke("reset_stats").catch(function (e) { console.error("reset_stats:", e); });
      document.getElementById("resetConfirm").style.display = "none";
      document.getElementById("resetStatsBtn").style.display = "";
    };
    document.getElementById("resetNo").onclick = function () {
      document.getElementById("resetConfirm").style.display = "none";
      document.getElementById("resetStatsBtn").style.display = "";
    };

    // Dashboard buttons
    document.getElementById("takeBreakBtn").onclick = function () {
      T.core.invoke("take_break_now").then(function () {
        return Promise.all([
          T.core.invoke("get_tip").catch(function () { return null; }),
          T.core.invoke("get_state")
        ]);
      }).then(function (results) {
        var tip = results[0], s = results[1];
        showBreak(tip, s.snooze_count >= s.max_snoozes);
      }).catch(function () { showBreak(null, false); });
    };

    document.getElementById("pauseBtn").onclick = function () {
      T.core.invoke(paused ? "resume_monitoring" : "pause_monitoring").catch(function (e) { console.error("pause/resume:", e); });
    };

    // Break controls
    document.getElementById("startBreakBtn").onclick = function () {
      if (bTimer) { clearInterval(bTimer); bTimer = null; }
      document.getElementById("startBreakBtn").style.display = "none";
      breakRem = breakDur;
      bTimer = setInterval(function () {
        breakRem--; updBreak();
        if (breakRem <= 0) {
          clearInterval(bTimer);
          T.core.invoke("finish_break").catch(function (e) { console.error("finish_break:", e); });
          // Play completion sound
          T.core.invoke("play_complete_sound").catch(function (e) { console.error("play_complete_sound:", e); });
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
      T.core.invoke("skip_break").catch(function (e) { console.error("skip_break:", e); });
      hideBreak();
    };

    // Interval +/- (both settings panel and main panel)
    // Snap to next/prev multiple of step
    function snapUp(val, step, max) { return Math.min(max, Math.floor(val / step) * step + step); }
    function snapDown(val, step, min) { return Math.max(min, Math.ceil(val / step) * step - step); }

    function intUp() { curInt = snapUp(curInt, 5, 120); markDirty(); refreshSettingsUI(); debounce("int", function () { T.core.invoke("set_interval", { minutes: curInt }); }, 300); }
    function intDown() { curInt = snapDown(curInt, 5, 5); markDirty(); refreshSettingsUI(); debounce("int", function () { T.core.invoke("set_interval", { minutes: curInt }); }, 300); }
    function brkUp() { curBrk = snapUp(curBrk, 5, 60); markDirty(); refreshSettingsUI(); debounce("brk", function () { T.core.invoke("set_break_duration", { seconds: curBrk }); }, 300); }
    function brkDown() { curBrk = snapDown(curBrk, 5, 5); markDirty(); refreshSettingsUI(); debounce("brk", function () { T.core.invoke("set_break_duration", { seconds: curBrk }); }, 300); }

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
      curSnzDur = Math.min(curInt - 1, curSnzDur + 1); markDirty(); refreshSettingsUI();
      debounce("snzDur", function () { T.core.invoke("set_snooze_duration", { minutes: curSnzDur }); }, 300);
    };
    document.getElementById("snzDurDown").onclick = function () {
      curSnzDur = Math.max(1, curSnzDur - 1); markDirty(); refreshSettingsUI();
      debounce("snzDur", function () { T.core.invoke("set_snooze_duration", { minutes: curSnzDur }); }, 300);
    };
    // Max snoozes +/-
    document.getElementById("snzMaxUp").onclick = function () {
      curSnzMax = Math.min(10, curSnzMax + 1); markDirty(); refreshSettingsUI();
      debounce("snzMax", function () { T.core.invoke("set_max_snoozes", { count: curSnzMax }); }, 300);
    };
    document.getElementById("snzMaxDown").onclick = function () {
      curSnzMax = Math.max(1, curSnzMax - 1); markDirty(); refreshSettingsUI();
      debounce("snzMax", function () { T.core.invoke("set_max_snoozes", { count: curSnzMax }); }, 300);
    };

    // Click-to-edit: Interval (settings + dashboard)
    var intEditOpts = {
      getValue: function () { return curInt; },
      setValue: function (v) { curInt = v; markDirty(); T.core.invoke("set_interval", { minutes: v }); },
      min: 5, max: 120,
      format: function (v) { return v + " min"; },
      colorClass: ""
    };
    makeEditable("intVal", intEditOpts);
    makeEditable("intValMain", intEditOpts);

    // Click-to-edit: Break duration (settings + dashboard)
    var brkEditOpts = {
      getValue: function () { return curBrk; },
      setValue: function (v) { curBrk = v; markDirty(); T.core.invoke("set_break_duration", { seconds: v }); },
      min: 5, max: 60,
      format: function (v) { return v + " sec"; },
      colorClass: "pink"
    };
    makeEditable("brkVal", brkEditOpts);
    makeEditable("brkValMain", brkEditOpts);

    // Click-to-edit: Snooze duration
    makeEditable("snzDurVal", {
      getValue: function () { return curSnzDur; },
      setValue: function (v) { curSnzDur = v; markDirty(); T.core.invoke("set_snooze_duration", { minutes: v }); },
      min: 1, max: 59,
      format: function (v) { return v + " min"; }
    });

    // Click-to-edit: Max snoozes
    makeEditable("snzMaxVal", {
      getValue: function () { return curSnzMax; },
      setValue: function (v) { curSnzMax = v; markDirty(); T.core.invoke("set_max_snoozes", { count: v }); },
      min: 1, max: 10,
      format: function (v) { return String(v); }
    });

    // Dashboard mode chips
    document.querySelectorAll(".chip").forEach(function (c) {
      c.onclick = function () { T.core.invoke("set_reminder_mode", { mode: c.dataset.mode }); };
    });

    // Settings: Timer in menu bar toggle
    document.getElementById("timerMenuToggle").onclick = function () {
      T.core.invoke("toggle_timer_in_menu").catch(function (e) { console.error("toggle_timer_in_menu:", e); });
    };

    // Settings: Reminder modes
    document.querySelectorAll(".mode-row").forEach(function (r) {
      r.onclick = function () { T.core.invoke("set_reminder_mode", { mode: r.dataset.mode }).catch(function (e) { console.error("set_reminder_mode:", e); }); };
    });

    // Settings: Auto-start toggle
    document.getElementById("autoStartToggle").onclick = function () {
      var isOn = this.classList.contains("on");
      T.core.invoke("set_auto_start", { enabled: !isOn }).catch(function (e) { console.error("set_auto_start:", e); });
    };

    document.getElementById("soundToggle").onclick = function () {
      T.core.invoke("toggle_sound").catch(function (e) { console.error("toggle_sound:", e); });
    };

    // Header sound button
    document.getElementById("soundBtn").onclick = function () {
      T.core.invoke("toggle_sound").catch(function (e) { console.error("toggle_sound:", e); });
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
      if (e.key === "Escape" || (e.key === "Backspace" && e.target.tagName !== "INPUT")) {
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
