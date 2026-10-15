/**
 * Brill dailies → Google Drive.
 *
 * A Google Apps Script (script.google.com) that saves each Brill
 * Bridge Challenge "daily event results" mail to a Drive folder: the
 * mail's field PBNs (one per event) plus its text as mail.txt, one
 * subfolder per day. rusty-bidding-bot reads that folder through the
 * Google Drive connector and runs probes/tools/brill_report.py on it.
 *
 * Install (once):
 *   1. https://script.google.com → New project, name it "Brill dailies".
 *   2. Replace Code.gs with this file and save.
 *   3. Run `install` (Run ▸ install). Google asks for permission to read
 *      Gmail, change Gmail labels and write Drive files: allow it.
 *      `install` makes a daily trigger and runs `saveBrillDailies` once,
 *      which also saves every results mail already in the mailbox.
 *
 * What it does on each run:
 *   - Finds mails from the Brill daily sender without the label
 *     "brill/saved".
 *   - For each, reads the date from the subject ("Rusty — 2026-10-14 —
 *     daily event results"), makes Drive folder
 *     "Brill dailies/2026-10-14/", and writes every .pbn attachment and
 *     mail.txt there. A file that is already there is left alone.
 *   - Labels the thread "brill/saved", so it is not read again.
 *
 * Nothing is deleted, forwarded or sent. To stop: Triggers (clock icon)
 * ▸ delete the trigger, or run `uninstall`.
 *
 * Fetching the files onto the Mac (optional, recommended): the script
 * can also serve the folder to probes/tools/brill_fetch.py, so the files
 * land in .rbb-cache/brill/<day>/ without passing through anything else.
 *   4. Run `makeToken` once: it stores a random token and logs it
 *      (View ▸ Logs). Keep it private.
 *   5. Deploy ▸ New deployment ▸ type "Web app": execute as "Me",
 *      who has access "Anyone". Copy the web app URL.
 *   6. On the Mac, put the URL and the token in
 *      ~/.config/rusty-bidding-bot/brill-drive.json:
 *        {"url": "https://script.google.com/macros/s/.../exec", "token": "..."}
 *      then run `python3 probes/tools/brill_fetch.py`.
 * Without the token the web app answers nothing; it only lists and
 * returns files under "Brill dailies".
 */

const SENDER = 'daily@brillchallenge.snippen.dk';
const ROOT_FOLDER = 'Brill dailies';
const DONE_LABEL = 'brill/saved';
// The mail comes at about 05:00 UTC; run a little after.
const RUN_AT_HOUR_UTC = 6;

function install() {
  uninstall();
  ScriptApp.newTrigger('saveBrillDailies')
    .timeBased()
    .atHour(RUN_AT_HOUR_UTC)
    .everyDays(1)
    .inTimezone('Etc/UTC')
    .create();
  saveBrillDailies();
}

function uninstall() {
  for (const t of ScriptApp.getProjectTriggers()) {
    if (t.getHandlerFunction() === 'saveBrillDailies') ScriptApp.deleteTrigger(t);
  }
}

function saveBrillDailies() {
  const label = GmailApp.getUserLabelByName(DONE_LABEL) || GmailApp.createLabel(DONE_LABEL);
  const root = folderNamed_(DriveApp.getRootFolder(), ROOT_FOLDER);
  const threads = GmailApp.search(`from:${SENDER} has:attachment -label:${DONE_LABEL.replace('/', '-')}`, 0, 50);
  let saved = 0;
  for (const thread of threads) {
    for (const msg of thread.getMessages()) {
      if (msg.getFrom().indexOf(SENDER) < 0) continue;
      const day = dayOf_(msg);
      const folder = folderNamed_(root, day);
      for (const att of msg.getAttachments()) {
        const name = att.getName();
        if (!/\.pbn$/i.test(name)) continue;
        if (!folder.getFilesByName(name).hasNext()) {
          folder.createFile(att.copyBlob().setName(name).setContentType('text/plain'));
          saved++;
        }
      }
      if (!folder.getFilesByName('mail.txt').hasNext()) {
        folder.createFile('mail.txt', msg.getPlainBody(), MimeType.PLAIN_TEXT);
      }
    }
    thread.addLabel(label);
  }
  console.log(`saved ${saved} PBN file(s) from ${threads.length} mail(s)`);
}

/** The day the results are for, from the subject, else the mail's date. */
function dayOf_(msg) {
  const m = msg.getSubject().match(/(\d{4}-\d{2}-\d{2})/);
  if (m) return m[1];
  return Utilities.formatDate(msg.getDate(), 'Etc/UTC', 'yyyy-MM-dd');
}

function folderNamed_(parent, name) {
  const it = parent.getFoldersByName(name);
  return it.hasNext() ? it.next() : parent.createFolder(name);
}

/** Stores a random access token for the web app and logs it. */
function makeToken() {
  const token = Utilities.getUuid().replace(/-/g, '') + Utilities.getUuid().replace(/-/g, '');
  PropertiesService.getScriptProperties().setProperty('TOKEN', token);
  console.log('token: ' + token);
}

/**
 * The web app: ?token=..&list=1 → {days: {"2026-10-14": [{name, id, size}]}};
 * ?token=..&id=<file id> → the file's text. Only files under ROOT_FOLDER.
 */
function doGet(e) {
  const p = (e && e.parameter) || {};
  const token = PropertiesService.getScriptProperties().getProperty('TOKEN');
  if (!token || p.token !== token) return text_('forbidden');
  const roots = DriveApp.getRootFolder().getFoldersByName(ROOT_FOLDER);
  if (!roots.hasNext()) return json_({days: {}});
  const root = roots.next();
  if (p.list) {
    const days = {};
    const it = root.getFolders();
    while (it.hasNext()) {
      const f = it.next();
      const files = [];
      const fi = f.getFiles();
      while (fi.hasNext()) {
        const x = fi.next();
        files.push({name: x.getName(), id: x.getId(), size: x.getSize()});
      }
      days[f.getName()] = files;
    }
    return json_({days: days});
  }
  if (p.id) {
    const file = DriveApp.getFileById(p.id);
    // Only serve files inside a day folder under ROOT_FOLDER.
    const parents = file.getParents();
    while (parents.hasNext()) {
      const day = parents.next();
      const up = day.getParents();
      while (up.hasNext()) {
        if (up.next().getId() === root.getId()) return text_(file.getBlob().getDataAsString());
      }
    }
    return text_('forbidden');
  }
  return text_('usage: ?token=..&list=1 or ?token=..&id=<file id>');
}

function json_(o) {
  return ContentService.createTextOutput(JSON.stringify(o)).setMimeType(ContentService.MimeType.JSON);
}

function text_(s) {
  return ContentService.createTextOutput(s).setMimeType(ContentService.MimeType.TEXT);
}
