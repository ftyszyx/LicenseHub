export function formatTime(iso?: string | null) {
    if (!iso) return '-';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso as string;
    // get local time
    return `${d.getFullYear()}-${(d.getMonth() + 1)}-${d.getDate()} ${d.getHours()}:${d.getMinutes()}:${d.getSeconds()}`
}