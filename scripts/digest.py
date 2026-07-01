# -*- coding: utf-8 -*-
"""Digest builder + emailer for the Bearings feed reader.

Email transport is optional and chosen by env vars (graceful no-op if unset):
  RESEND_API_KEY (+ DIGEST_FROM)              -> send via Resend HTTP API
  DIGEST_SMTP_HOST/PORT/USER/PASS (+ FROM)    -> send via SMTP (STARTTLS)
  DIGEST_TO                                   -> recipient (default ursasteward@pm.me)
Always writes the digest to /opt/bearings-rs/logs/ so it is observable even with no email.
"""
import os, json, smtplib
from email.message import EmailMessage
from urllib.request import Request, urlopen


def build_digest(ts, stats, total_new, total_past, pending_count, gaps,
                 archived=None, predictions=None, pending_th=0, agents_7d=None):
    day = ts[:10]
    L = [f"Bearings nightly research digest — {day} (UTC)", ""]
    L.append(f"New candidates queued:  {total_new}")
    L.append(f"Past-dated skipped:     {total_past}")
    L.append(f"Pending review queue:   {pending_count} events, {pending_th} titleholder proposals")
    L.append(f"Title-holder gaps:      {len(gaps)}")
    L.append(f"Archived (date passed): {len(archived) if archived else 0}")
    L.append(f"Likely repeats ahead:   {len(predictions) if predictions else 0}")
    L += ["", "Per feed:"]
    for s in stats:
        err = f"   ERROR: {s['error']}" if s.get("error") else ""
        L.append(f"  - {s['org']}: {s['parsed']} parsed, {s['new']} new, {s['past']} past{err}")
    if gaps:
        L += ["", "Competitions missing a title holder:"]
        for g in gaps:
            L.append(f"  - [{g.get('scope','')}] {g.get('name','')} ({g.get('country','')})")
    if archived:
        L += ["", f"Archived now-past events ({len(archived)}):"]
        for a in archived:
            L.append(f"  - {a.get('name','')}")
    if predictions:
        L += ["", f"Likely to repeat — no confirmed edition yet ({len(predictions)}); consider outreach:"]
        for p in predictions:
            L.append(f"  - {p.get('sample_name','')} ({p.get('city','')}) "
                     f"~ {p.get('predicted_date','')} [{p.get('confidence','')}]")
    if agents_7d:
        # Agent-fleet health: what the agents did in the last 7 days (from agent_actions),
        # so the fleet stays observable and the review queue never silently piles up.
        total = sum(agents_7d.values())
        L += ["", f"Agents (last 7 days) — {total} action(s):"]
        for action, n in sorted(agents_7d.items(), key=lambda kv: (-kv[1], kv[0])):
            L.append(f"  - {action}: {n}")
    if pending_count + pending_th >= 40:
        L += ["", f"⚠ Review backlog is building ({pending_count + pending_th} pending) — "
                  "worth a review pass."]
    L += ["", "Review queue: https://srv1744879.hstgr.cloud/?zone=admin&token=<ADMIN_TOKEN>"]
    body = "\n".join(L)
    subject = f"[Bearings] research digest {day}: {total_new} new, {pending_count + pending_th} pending"
    return {"subject": subject, "body": body}


def write_log(digest):
    logdir = "/opt/bearings-rs/logs"
    os.makedirs(logdir, exist_ok=True)
    with open(f"{logdir}/digest.log", "a", encoding="utf-8") as f:
        f.write("\n" + "=" * 64 + "\n" + digest["body"] + "\n")
    with open(f"{logdir}/digest-latest.txt", "w", encoding="utf-8") as f:
        f.write(digest["body"] + "\n")
    print(f"[digest] written to {logdir}/digest.log")


def send_digest(digest):
    to_addr   = os.environ.get("DIGEST_TO", "ursasteward@pm.me")
    from_addr = os.environ.get("DIGEST_FROM", "")
    subject, body = digest["subject"], digest["body"]
    resend_key = os.environ.get("RESEND_API_KEY")
    smtp_host  = os.environ.get("DIGEST_SMTP_HOST")

    if resend_key:
        try:
            payload = json.dumps({"from": from_addr or "onboarding@resend.dev",
                                  "to": [to_addr], "subject": subject, "text": body}).encode()
            req = Request("https://api.resend.com/emails", data=payload,
                          headers={"Authorization": f"Bearer {resend_key}",
                                   "Content-Type": "application/json",
                                   # Resend's API is behind Cloudflare, which 403s the
                                   # default Python-urllib UA (error 1010) — set a real one.
                                   "User-Agent": "Bearings-Digest/1.0 (+https://bearings.community)"},
                          method="POST")
            with urlopen(req, timeout=20) as r:
                print(f"[digest] emailed via Resend ({r.status}) to {to_addr}")
            return
        except Exception as e:
            print(f"[digest] Resend send failed: {e}")

    if smtp_host:
        try:
            msg = EmailMessage()
            msg["Subject"] = subject
            msg["From"] = from_addr or os.environ.get("DIGEST_SMTP_USER", "bearings@localhost")
            msg["To"] = to_addr
            msg.set_content(body)
            port = int(os.environ.get("DIGEST_SMTP_PORT", "587"))
            user = os.environ.get("DIGEST_SMTP_USER")
            pw   = os.environ.get("DIGEST_SMTP_PASS")
            with smtplib.SMTP(smtp_host, port, timeout=20) as s:
                s.starttls()
                if user and pw:
                    s.login(user, pw)
                s.send_message(msg)
            print(f"[digest] emailed via SMTP to {to_addr}")
            return
        except Exception as e:
            print(f"[digest] SMTP send failed: {e}")

    print("[digest] no email transport set (RESEND_API_KEY or DIGEST_SMTP_*); wrote to log only")
