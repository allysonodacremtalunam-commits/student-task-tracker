// This file is the only JavaScript in the project. It calls the Rust/Axum
// JSON API (under /api/...) and updates the page. No frontend framework.

const taskListEl = document.getElementById('task-list');
const emptyMessageEl = document.getElementById('empty-message');
const historyListEl = document.getElementById('history-list');
const historyEmptyMessageEl = document.getElementById('history-empty-message');
const messageEl = document.getElementById('message');

const tabTasksBtn = document.getElementById('tab-tasks');
const tabHistoryBtn = document.getElementById('tab-history');
const tasksSection = document.getElementById('tasks-section');
const historySection = document.getElementById('history-section');

const searchInput = document.getElementById('search-input');
const statusFilter = document.getElementById('status-filter');
const priorityFilter = document.getElementById('priority-filter');

const addTaskBtn = document.getElementById('add-task-btn');
const addTaskModal = document.getElementById('add-task-modal');
const cancelAddBtn = document.getElementById('cancel-add-btn');
const addTaskForm = document.getElementById('add-task-form');
const formError = document.getElementById('form-error');

const viewTaskModal = document.getElementById('view-task-modal');
const viewTaskContent = document.getElementById('view-task-content');

function showMessage(text) {
  messageEl.textContent = text;
  messageEl.classList.add('visible');
  setTimeout(() => messageEl.classList.remove('visible'), 4000);
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function formatDateTime(value) {
  if (!value) return '-';
  const date = new Date(value);
  if (isNaN(date)) return value;
  return date.toLocaleString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  });
}

function isOverdue(task) {
  if (task.status !== 'Pending' || !task.due_at) return false;
  return new Date(task.due_at) < new Date();
}

// --- Tabs ---

tabTasksBtn.addEventListener('click', () => {
  tabTasksBtn.classList.add('active');
  tabHistoryBtn.classList.remove('active');
  tasksSection.classList.remove('hidden');
  historySection.classList.add('hidden');
  loadTasks();
});

tabHistoryBtn.addEventListener('click', () => {
  tabHistoryBtn.classList.add('active');
  tabTasksBtn.classList.remove('active');
  historySection.classList.remove('hidden');
  tasksSection.classList.add('hidden');
  loadHistory();
});

// --- Tasks table ---

function buildQueryString() {
  const params = new URLSearchParams();
  if (searchInput.value.trim()) params.set('search', searchInput.value.trim());
  if (statusFilter.value !== 'All') params.set('status', statusFilter.value);
  if (priorityFilter.value !== 'All') params.set('priority', priorityFilter.value);
  const query = params.toString();
  return query ? '?' + query : '';
}

async function loadTasks() {
  const response = await fetch('/api/tasks' + buildQueryString());
  const tasks = await response.json();
  renderTasks(tasks);
}

function renderTasks(tasks) {
  taskListEl.innerHTML = '';

  if (tasks.length === 0) {
    emptyMessageEl.style.display = 'block';
    return;
  }
  emptyMessageEl.style.display = 'none';

  for (const task of tasks) {
    const overdue = isOverdue(task);
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${task.id}</td>
      <td>${escapeHtml(task.title)}</td>
      <td>${escapeHtml(task.subject)}</td>
      <td><span class="badge priority-${task.priority.toLowerCase()}">${task.priority}</span></td>
      <td class="${overdue ? 'overdue-text' : ''}">${formatDateTime(task.due_at)}</td>
      <td>
        <span class="badge status-${task.status.toLowerCase()}">${task.status}</span>
        ${overdue ? '<span class="badge status-overdue">Overdue</span>' : ''}
      </td>
      <td>
        <button class="link-btn" onclick="viewTask(${task.id})">View</button>
        ${task.status === 'Pending' ? `<button class="link-btn" onclick="completeTask(${task.id}, false)">Complete</button>` : ''}
        <button class="link-btn danger" onclick="deleteTask(${task.id}, false)">Delete</button>
      </td>
    `;
    taskListEl.appendChild(row);
  }
}

// --- History table ---

async function loadHistory() {
  const response = await fetch('/api/tasks/history');
  const tasks = await response.json();
  renderHistory(tasks);
}

function renderHistory(tasks) {
  historyListEl.innerHTML = '';

  if (tasks.length === 0) {
    historyEmptyMessageEl.style.display = 'block';
    return;
  }
  historyEmptyMessageEl.style.display = 'none';

  for (const task of tasks) {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${task.id}</td>
      <td>${escapeHtml(task.title)}</td>
      <td>${escapeHtml(task.subject)}</td>
      <td>${formatDateTime(task.due_at)}</td>
      <td>${formatDateTime(task.completed_at)}</td>
    `;
    historyListEl.appendChild(row);
  }
}

// --- Stats ---

async function loadStats() {
  const response = await fetch('/api/stats');
  const stats = await response.json();
  document.getElementById('stat-total').textContent = stats.total;
  document.getElementById('stat-pending').textContent = stats.pending;
  document.getElementById('stat-completed').textContent = stats.completed;
  document.getElementById('stat-high').textContent = stats.high_priority;
}

function refresh() {
  loadTasks();
  loadStats();
}

// --- Add Task ---

addTaskBtn.addEventListener('click', () => {
  addTaskForm.reset();
  formError.classList.add('hidden');
  addTaskModal.classList.remove('hidden');
});

cancelAddBtn.addEventListener('click', () => {
  addTaskModal.classList.add('hidden');
});

addTaskForm.addEventListener('submit', async (event) => {
  event.preventDefault();

  const payload = {
    title: document.getElementById('form-title').value,
    description: document.getElementById('form-description').value,
    subject: document.getElementById('form-subject').value,
    priority: document.getElementById('form-priority').value,
    due_at: document.getElementById('form-due').value,
  };

  const response = await fetch('/api/tasks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });

  if (response.ok) {
    addTaskModal.classList.add('hidden');
    showMessage('Task added successfully.');
    refresh();
  } else {
    const body = await response.json().catch(() => null);
    formError.textContent = (body && body.message) || 'Could not save the task.';
    formError.classList.remove('hidden');
  }
});

// --- View Task ---

async function viewTask(id) {
  const response = await fetch('/api/tasks/' + id);
  if (!response.ok) {
    showMessage('That task could not be found.');
    return;
  }
  const task = await response.json();

  viewTaskContent.innerHTML = `
    <h2>${escapeHtml(task.title)}</h2>
    <p><strong>ID:</strong> ${task.id}</p>
    <p><strong>Subject:</strong> ${escapeHtml(task.subject)}</p>
    <p><strong>Priority:</strong> ${task.priority}</p>
    <p><strong>Status:</strong> ${task.status}</p>
    <p><strong>Due:</strong> ${formatDateTime(task.due_at)}</p>
    <p><strong>Created:</strong> ${task.created_at}</p>
    ${task.completed_at ? `<p><strong>Completed On:</strong> ${formatDateTime(task.completed_at)}</p>` : ''}
    <p><strong>Description:</strong><br>${escapeHtml(task.description)}</p>
    <div class="modal-actions">
      ${task.status === 'Pending' ? `<button class="btn btn-primary" onclick="completeTask(${task.id}, true)">Mark as Completed</button>` : ''}
      <button class="btn danger" onclick="deleteTask(${task.id}, true)">Delete</button>
      <button class="btn" onclick="closeViewModal()">Back to Tasks</button>
    </div>
  `;
  viewTaskModal.classList.remove('hidden');
}

function closeViewModal() {
  viewTaskModal.classList.add('hidden');
}

// --- Complete Task ---

async function completeTask(id, fromModal) {
  const response = await fetch(`/api/tasks/${id}/complete`, { method: 'POST' });
  if (response.ok) {
    showMessage('Task marked as completed.');
    if (fromModal) closeViewModal();
    refresh();
  } else {
    showMessage('Could not update that task.');
  }
}

// --- Delete Task ---

async function deleteTask(id, fromModal) {
  if (!confirm('Are you sure you want to delete this task?')) return;

  const response = await fetch(`/api/tasks/${id}`, { method: 'DELETE' });
  if (response.ok) {
    showMessage('Task deleted.');
    if (fromModal) closeViewModal();
    refresh();
  } else {
    showMessage('Could not delete that task.');
  }
}

// --- Search & Filters ---

searchInput.addEventListener('input', loadTasks);
statusFilter.addEventListener('change', loadTasks);
priorityFilter.addEventListener('change', loadTasks);

// ============================================================
// Deadline reminders
//
// This checks pending tasks every minute while this browser tab is open.
// It fires a reminder 30 minutes before a task is due, and a separate
// "late" notice once the due time has passed. This only works while the
// tab stays open — it is NOT a real push notification service, since that
// would need a background service worker plus a server that can wake your
// device, which is outside the scope of this project.
// ============================================================

const NOTIFIED_KEY = 'taskTrackerNotifiedIds';

function getNotifiedIds() {
  try {
    return new Set(JSON.parse(localStorage.getItem(NOTIFIED_KEY)) || []);
  } catch {
    return new Set();
  }
}

function saveNotifiedIds(ids) {
  localStorage.setItem(NOTIFIED_KEY, JSON.stringify([...ids]));
}

function notifyUser(title, body) {
  if ('Notification' in window && Notification.permission === 'granted') {
    new Notification(title, { body });
  }
  showMessage(body);
}

async function checkDeadlines() {
  const response = await fetch('/api/tasks?status=Pending');
  if (!response.ok) return;
  const tasks = await response.json();

  const notified = getNotifiedIds();
  const now = new Date();
  let changed = false;

  for (const task of tasks) {
    if (!task.due_at) continue;
    const due = new Date(task.due_at);
    if (isNaN(due)) continue;

    const minutesLeft = (due - now) / 60000;
    const soonKey = `soon-${task.id}`;
    const lateKey = `late-${task.id}`;

    if (minutesLeft <= 30 && minutesLeft > 0 && !notified.has(soonKey)) {
      notifyUser('Task Reminder', `"${task.title}" is due in about ${Math.max(1, Math.round(minutesLeft))} minute(s). Better start now!`);
      notified.add(soonKey);
      changed = true;
    }

    if (minutesLeft <= 0 && !notified.has(lateKey)) {
      notifyUser('Task Overdue', `"${task.title}" is now overdue — you're going to be late to pass it!`);
      notified.add(lateKey);
      changed = true;
    }
  }

  if (changed) saveNotifiedIds(notified);
}

if ('Notification' in window && Notification.permission === 'default') {
  Notification.requestPermission();
}

setInterval(checkDeadlines, 60000);
checkDeadlines();

// Initial load when the page opens.
refresh();
