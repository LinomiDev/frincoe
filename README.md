# frincoe

A messaging bus written in Rust.

The name `frincoe` comes from a composition and abbreviation of
the Latin word fringo(split in English), the origin of diffringo(diffract in English),
and the Japanese word koe(voice in English);
pronounced in Classical Latin way, as f-rin-co-ae.
BTW, sound waves (i.e. voice) in real world can't diffract.

Goals (i.e. traits we desire but not achieved yet):

- Flexible: Every component can be replaced by another implement.
  And only the module directly assembling the component into the frame
  is needed to be modified to replace the implement.
  Furthermore, a goal is to make it possible to assembly modules in configurations,
  without any overhead.
- Efficient: Introduce least overhead as possible;
  at the best case, there shouldn't be any cost besides iterating the clients
  and collecting the result.
- Friendly: Well-documented, well-formatted source code;
  interfaces as simple, clear and consistent as possible;
  and every detail is covered by the document.

## License

See [LICENSE](./LICENSE) for the complete license.

> This program is free software: you can redistribute it and/or modify
> it under the terms of the GNU Affero General Public License as published
> by the Free Software Foundation, either version 3 of the License, or
> (at your option) any later version.
>
> This program is distributed in the hope that it will be useful,
> but WITHOUT ANY WARRANTY; without even the implied warranty of
> MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
> GNU Affero General Public License for more details.
>
> You should have received a copy of the GNU Affero General Public License
> along with this program.  If not, see <https://www.gnu.org/licenses/>.

[//modeline]: random:// " vim:set spell nofoldenable: "
