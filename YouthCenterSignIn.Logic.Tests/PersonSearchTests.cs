using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.Linq;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class PersonSearchTests : TestBase
    {
        [TestMethod]
        public void PersonSearch_ResultsTest()
        {
            var search = new PersonSearch
            {
                SearchText = "James"
            };

            DataProvider.RunningTask?.Wait();
            Assert.AreEqual("James", search.Single().FirstName,
                "The search text was set to James, so the first (and only) result should have a first name of James.");
        }
    }
}
