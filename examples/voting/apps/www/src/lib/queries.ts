export class QueryKeyFactory {
  static proposals() {
    return ["proposals"];
  }

  static proposalCount() {
    return ["proposal-count"];
  }

  static proposal(proposalId?: number | string, account?: `0x${string}`) {
    return ["proposal", proposalId, account];
  }
}
