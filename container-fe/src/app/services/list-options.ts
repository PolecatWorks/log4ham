export interface ListOptions {
    offset?: Number;
    limit?: Number;
}

export interface ListIds {
    ids: Number[];
    options: ListOptions;
}
