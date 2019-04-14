using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    public class TestDataProvider : DataProvider
    {
        public override Task ShowMessage(string message = null, Exception ex = null)
        {
            throw new MessageException(message, ex);
        }

        #region People

        internal List<Person> People { get; } = new List<Person>
        {
            new Person(
                id: "ESHJAM",
                firstName: "James",
                lastName: "Esh",
                notes: new Guardian("Glenn Esh", "(717) 629-0658", "gesh@eshcom.com").ToString(),
                birthDate: new DateTimeOffset(new DateTime(1999, 2, 23)),
                address: new Address("52 Evergreen", "Gordonville", "PA")),
            new Person(
                id: "ESHDER",
                firstName: "Derek",
                lastName: "Esh",
                notes: new Guardian("Glenn Esh", "(717) 629-0658", "gesh@eshcom.com").ToString(),
                birthDate: new DateTimeOffset(new DateTime(2006, 9, 24)),
                address: new Address("52 Evergreen", "Gordonville", "PA")),
            new Person(
                id: "BEACHR",
                firstName: "Chris",
                lastName: "Beachy",
                notes: null,
                birthDate: new DateTimeOffset(),
                address: null),
            new Person(
                id: "GLIDAN",
                firstName: "Daniel",
                lastName: "Glick",
                notes: new Guardian("Dale Glick", "717-348-0236").ToString(),
                birthDate: new DateTimeOffset(new DateTime(1999, 7, 12)),
                address: new Address(null, "Gap", "PA")),
            new Person(
                id: "PETMER",
                firstName: "Merv",
                lastName: "Petershiem",
                notes: "",
                birthDate: new DateTimeOffset(new DateTime(1975, 12, 3)),
                address: new Address("4610 Horseshoe Pike", "Honey Brook", "PA")),
            new Person(
                id: "MCKPET",
                firstName: "Peter",
                lastName: "McKinnin",
                notes: "",
                birthDate: new DateTimeOffset(new DateTime(1985, 7, 30)),
                address: new Address("4610 Horseshoe Pike", "Seattle", "Washington")),
        };

        public override Task<List<Person>> GetPeople()
        {
            return Task.FromResult(People.ToList());
        }

        public override Task<string> SavePerson(Person person)
        {
            People.Add(person);

            return Task.FromResult(Guid.NewGuid().ToString());
        }

        #endregion

        #region Json

        #region Files

        Dictionary<string, string> JsonFiles { get; } = new Dictionary<string, string>();

        protected override Task<string> GetJsonFileContent(string file)
        {
            if (JsonFiles.TryGetValue(file, out string json))
                return Task.FromResult(json);

            return Task.FromResult("");
        }

        protected override Task SetJsonFileContent(string file, string json)
        {
            if (JsonFiles.ContainsKey(file))
                JsonFiles[file] = json;
            else
                JsonFiles.Add(file, json);

            return Task.CompletedTask;
        }

        #endregion

        #region Settings

        Dictionary<string, string> JsonSettings { get; } = new Dictionary<string, string>();

        protected override string GetJsonSetting(string key)
        {
            if (JsonSettings.TryGetValue(key, out string json))
                return json;

            return "";
        }

        protected override void SetJsonSetting(string key, string json)
        {
            if (JsonSettings.ContainsKey(key))
                JsonSettings[key] = json;
            else
                JsonSettings.Add(key, json);
        }

        #endregion

        #endregion
    }

    public class MessageException : Exception
    {
        public MessageException(string message, Exception exception) : base(message, exception)
        { }
    }
}
