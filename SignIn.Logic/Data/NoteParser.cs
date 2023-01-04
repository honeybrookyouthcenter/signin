using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Xml.Linq;

namespace SignIn.Logic.Data
{
    public static class NoteParser
    {
        public static string GetLabelExpression(string label) => $@"(?<={label}).*";

        public static string ToNotes(params (string label, string value)[] values) => JoinValues(values.Select(p => $"{p.label}{p.value}"));

        public static string JoinValues(IEnumerable<string> values) => string.Join("\r\n", values);

        public static IEnumerable<string> FromNotes(string notes, params Regex[] labelSearches)
        {
            if (notes == null)
                notes = "";

            foreach (var regex in labelSearches)
                yield return regex.Match(notes).Value.Trim();
        }
    }
}
