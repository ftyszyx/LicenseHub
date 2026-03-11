export function formatTime(value?: string | number | null) {
    if (value == null || value === '') return '-';
    const dateValue = typeof value === 'number'
        ? new Date(value < 1_000_000_000_000 ? value * 1000 : value)
        : new Date(value);
    if (Number.isNaN(dateValue.getTime())) return String(value);
    // get local time
    return `${dateValue.getFullYear()}-${(dateValue.getMonth() + 1)}-${dateValue.getDate()} ${dateValue.getHours()}:${dateValue.getMinutes()}:${dateValue.getSeconds()}`
}
