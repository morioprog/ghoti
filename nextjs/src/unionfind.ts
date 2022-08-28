export class UnionFind {
  private parents: number[];

  public constructor(private n: number) {
    this.parents = new Array<number>(n).fill(-1);
  }

  public find(a: number): number {
    if (this.parents[a] < 0) {
      return a;
    }
    return (this.parents[a] = this.find(this.parents[a]));
  }

  public size(a: number): number {
    return -this.parents[this.find(a)];
  }

  public unite(a: number, b: number): boolean {
    let ra = this.find(a);
    let rb = this.find(b);
    if (ra === rb) {
      return false;
    }

    if (this.size(ra) < this.size(rb)) {
      [ra, rb] = [rb, ra];
    }
    this.parents[ra] += this.parents[rb];
    this.parents[rb] = ra;
    return true;
  }

  public same(a: number, b: number): boolean {
    const ra = this.find(a);
    const rb = this.find(b);
    return ra === rb;
  }
}
